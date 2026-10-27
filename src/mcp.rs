use std::{collections::BTreeMap, sync::Arc};

use actix_web::{error::ErrorInternalServerError, http::header, web, HttpRequest, HttpResponse};
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
};
use serde::Deserialize;
use serde_json::json;
use time::{format_description::well_known::Iso8601, Date, Duration, OffsetDateTime};

use crate::{route::model::ExchangeRate, service::provider};

#[async_trait]
trait RateSource: Send + Sync + 'static {
    async fn symbols(&self) -> std::collections::HashMap<String, String>;
    async fn rates(&self, base: String) -> ExchangeRate;
    async fn history(
        &self,
        base: String,
        from: Date,
        to: Date,
    ) -> std::collections::HashMap<Date, ExchangeRate>;
}

struct Providers;

#[async_trait]
impl RateSource for Providers {
    async fn symbols(&self) -> std::collections::HashMap<String, String> {
        provider::symbols().await
    }
    async fn rates(&self, base: String) -> ExchangeRate {
        provider::rates_of(base).await
    }
    async fn history(
        &self,
        base: String,
        from: Date,
        to: Date,
    ) -> std::collections::HashMap<Date, ExchangeRate> {
        provider::historical_rates_of(base, from, to).await
    }
}

#[derive(Clone)]
struct ExchangeMcp {
    source: Arc<dyn RateSource>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RateRequest {
    /// Three-letter base currency code, such as CHF.
    base: String,
    /// Optional three-letter counter currency code, such as EUR.
    counter: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct HistoryRequest {
    /// Three-letter base currency code, such as CHF.
    base: String,
    /// Optional three-letter counter currency code, such as EUR.
    counter: Option<String>,
    /// First date, inclusive, in YYYY-MM-DD format.
    from: String,
    /// Last date, inclusive, in YYYY-MM-DD format. At most 31 days may be requested.
    to: String,
}

fn currency(value: &str) -> Result<String, String> {
    if value.len() == 3 && value.bytes().all(|b| b.is_ascii_alphabetic()) {
        Ok(value.to_ascii_uppercase())
    } else {
        Err("Currency codes must contain exactly three ASCII letters".into())
    }
}

fn parse_pair(base: &str, counter: Option<&str>) -> Result<(String, Option<String>), String> {
    Ok((currency(base)?, counter.map(currency).transpose()?))
}

fn dates(from: &str, to: &str) -> Result<(Date, Date), String> {
    let parse = |value| {
        Date::parse(value, &Iso8601::DATE)
            .map_err(|_| "Dates must use YYYY-MM-DD format".to_string())
    };
    let from = parse(from)?;
    let to = parse(to)?;
    if from > to {
        return Err("The start date must be on or before the end date".into());
    }
    if to - from > Duration::days(30) {
        return Err("The date range cannot exceed 31 days".into());
    }
    if to > OffsetDateTime::now_utc().date() {
        return Err("The end date cannot be in the future".into());
    }
    Ok((from, to))
}

fn tool_error(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

#[tool_router(server_handler)]
impl ExchangeMcp {
    #[tool(description = "List supported currency codes and names")]
    async fn list_currencies(&self) -> CallToolResult {
        let currencies: BTreeMap<_, _> = self
            .source
            .symbols()
            .await
            .into_iter()
            .map(|(code, name)| (code.to_ascii_uppercase(), name))
            .collect();
        if currencies.is_empty() {
            return tool_error("No currencies are available from the providers");
        }
        CallToolResult::structured(json!({"currencies": currencies}))
    }

    #[tool(
        description = "Get current indicative exchange rates for a base currency and optionally a counter currency"
    )]
    async fn get_exchange_rates(
        &self,
        Parameters(args): Parameters<RateRequest>,
    ) -> CallToolResult {
        let (base, counter) = match parse_pair(&args.base, args.counter.as_deref()) {
            Ok(pair) => pair,
            Err(e) => return tool_error(e),
        };
        let rates = self.source.rates(base.clone()).await.rates;
        if rates.is_empty() {
            return tool_error(format!("No rates are available for {base}"));
        }
        match counter {
            Some(counter) => match rates.get(&counter) {
                Some(rate) => CallToolResult::structured(
                    json!({"base": base, "counter": counter, "rate": rate}),
                ),
                None => tool_error(format!("No rate is available for {base}/{counter}")),
            },
            None => CallToolResult::structured(json!({"base": base, "rates": rates})),
        }
    }

    #[tool(
        description = "Get historical indicative exchange rates over an inclusive date range of up to 31 days"
    )]
    async fn get_historical_rates(
        &self,
        Parameters(args): Parameters<HistoryRequest>,
    ) -> CallToolResult {
        let (base, counter) = match parse_pair(&args.base, args.counter.as_deref()) {
            Ok(pair) => pair,
            Err(e) => return tool_error(e),
        };
        let (from, to) = match dates(&args.from, &args.to) {
            Ok(range) => range,
            Err(e) => return tool_error(e),
        };
        let history = self.source.history(base.clone(), from, to).await;
        if let Some(counter) = counter {
            let rates: BTreeMap<_, _> = history
                .into_iter()
                .filter_map(|(date, rate)| {
                    rate.rates
                        .get(&counter)
                        .map(|value| (date.to_string(), *value))
                })
                .collect();
            if rates.is_empty() {
                return tool_error(format!(
                    "No historical rates are available for {base}/{counter} in the requested range"
                ));
            }
            CallToolResult::structured(json!({"base": base, "counter": counter, "rates": rates}))
        } else {
            let rates: BTreeMap<_, _> = history
                .into_iter()
                .filter(|(_, rate)| !rate.rates.is_empty())
                .map(|(date, rate)| (date.to_string(), rate.rates))
                .collect();
            if rates.is_empty() {
                return tool_error(format!(
                    "No historical rates are available for {base} in the requested range"
                ));
            }
            CallToolResult::structured(json!({"base": base, "rates": rates}))
        }
    }
}

type McpHttp = StreamableHttpService<ExchangeMcp, LocalSessionManager>;

const MAX_REQUEST_BYTES: usize = 256 * 1024;

fn http_service(source: Arc<dyn RateSource>) -> McpHttp {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(MAX_REQUEST_BYTES);
    StreamableHttpService::new(
        move || {
            Ok(ExchangeMcp {
                source: source.clone(),
            })
        },
        Arc::new(LocalSessionManager::default()),
        config,
    )
}

async fn handle(
    request: HttpRequest,
    body: web::Bytes,
    service: web::Data<McpHttp>,
) -> actix_web::Result<HttpResponse> {
    if request.headers().contains_key(header::ORIGIN) {
        return Ok(HttpResponse::Forbidden().finish());
    }
    if request.method() == actix_web::http::Method::GET
        && request
            .headers()
            .get(header::ACCEPT)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.split(',').any(|media_type| {
                    media_type.split(';').next().unwrap_or("").trim() == "text/html"
                })
            })
    {
        return Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header((header::VARY, "Accept"))
            .body(
                r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Exchange Rates MCP</title></head>
<body>
    <h1>Exchange Rates MCP</h1>
    <p>Copy this page's full URL into your MCP client's Streamable HTTP server settings.</p>
    <p>The endpoint accepts MCP requests via HTTP POST. Opening it in a browser displays this connection guide.</p>
    <h2>Available tools</h2>
    <ul>
        <li><code>list_currencies</code>: supported currency codes and names.</li>
        <li><code>get_exchange_rates</code>: current rates for a base currency and optional counter currency.</li>
        <li><code>get_historical_rates</code>: historical rates for an inclusive date range of up to 31 days.</li>
    </ul>
    <h2>Connect a client</h2>
    <p>Replace <code>MCP_URL</code> below with this page's full URL.</p>
    <pre>codex mcp add exchange-rates --url MCP_URL
gemini mcp add --transport http exchange-rates MCP_URL
claude mcp add --transport http exchange-rates MCP_URL</pre>
    <p><a href="/">Service home</a> · <a href="/docs/">REST API documentation</a></p>
</body>
</html>"#,
            ));
    }

    // Actix uses http 0.2; the MCP SDK uses http 1.x.
    let mut forwarded = http::Request::builder()
        .method(request.method().as_str())
        .uri(request.uri().to_string());
    for (name, value) in request.headers() {
        forwarded = forwarded.header(name.as_str(), value.as_bytes());
    }
    let forwarded = forwarded
        .body(Full::new(body))
        .map_err(ErrorInternalServerError)?;
    let (parts, body) = service.handle(forwarded).await.into_parts();
    let status = actix_web::http::StatusCode::from_u16(parts.status.as_u16())
        .map_err(ErrorInternalServerError)?;
    let mut response = HttpResponse::build(status);
    for (name, value) in &parts.headers {
        let name = header::HeaderName::from_bytes(name.as_str().as_bytes())
            .map_err(ErrorInternalServerError)?;
        let value =
            header::HeaderValue::from_bytes(value.as_bytes()).map_err(ErrorInternalServerError)?;
        response.append_header((name, value));
    }
    response.append_header((header::VARY, "Accept"));
    Ok(response.streaming(body.into_data_stream()))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config
        .app_data(web::Data::new(http_service(Arc::new(Providers))))
        .service(
            web::resource("/mcp")
                .app_data(web::PayloadConfig::new(MAX_REQUEST_BYTES))
                .to(handle),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{
        model::{CallToolRequestParams, ClientConfig},
        transport::StreamableHttpClientTransport,
        ServiceExt,
    };
    use std::collections::HashMap;
    use time::macros::date;

    struct TestRates;

    #[async_trait]
    impl RateSource for TestRates {
        async fn symbols(&self) -> HashMap<String, String> {
            HashMap::from([
                ("CHF".into(), "Swiss franc".into()),
                ("EUR".into(), "Euro".into()),
            ])
        }
        async fn rates(&self, base: String) -> ExchangeRate {
            ExchangeRate {
                base,
                rates: HashMap::from([("EUR".into(), 0.95), ("USD".into(), 1.1)]),
            }
        }
        async fn history(
            &self,
            base: String,
            _from: Date,
            _to: Date,
        ) -> HashMap<Date, ExchangeRate> {
            HashMap::from([(
                date!(2024 - 01 - 01),
                ExchangeRate {
                    base,
                    rates: HashMap::from([("EUR".into(), 0.95)]),
                },
            )])
        }
    }

    #[actix_web::test]
    async fn mcp_client_discovers_and_calls_tools() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let url = format!("http://{address}/mcp");
        let source = web::Data::new(http_service(Arc::new(TestRates)));
        let server = actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .configure(crate::route::routes::init_routes)
                .app_data(source.clone())
        })
        .workers(1)
        .listen(listener)
        .unwrap()
        .run();
        let server_handle = server.handle();
        actix_web::rt::spawn(server);
        let client = ClientConfig::default()
            .serve(StreamableHttpClientTransport::from_uri(url.clone()))
            .await
            .unwrap();

        let mut names: Vec<_> = client
            .list_all_tools()
            .await
            .unwrap()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        names.sort();
        assert_eq!(
            names,
            [
                "get_exchange_rates",
                "get_historical_rates",
                "list_currencies"
            ]
        );
        let call = |name: &'static str, args: serde_json::Value| {
            CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone())
        };
        let currencies = client
            .call_tool(call("list_currencies", json!({})))
            .await
            .unwrap();
        assert_eq!(
            currencies.structured_content.unwrap()["currencies"]["CHF"],
            "Swiss franc"
        );
        let rate = client
            .call_tool(call(
                "get_exchange_rates",
                json!({"base": "chf", "counter": "eur"}),
            ))
            .await
            .unwrap();
        assert_eq!(
            rate.structured_content.unwrap(),
            json!({"base": "CHF", "counter": "EUR", "rate": 0.95_f32})
        );
        let all = client
            .call_tool(call("get_exchange_rates", json!({"base": "CHF"})))
            .await
            .unwrap();
        assert!(all.structured_content.unwrap()["rates"]
            .get("USD")
            .is_some());
        let history = client
            .call_tool(call(
                "get_historical_rates",
                json!({"base": "CHF", "counter": "EUR", "from": "2024-01-01", "to": "2024-01-02"}),
            ))
            .await
            .unwrap();
        assert_eq!(
            history.structured_content.unwrap()["rates"]["2024-01-01"],
            json!(0.95_f32)
        );
        for (name, args) in [
            ("get_exchange_rates", json!({"base": "../CHF"})),
            (
                "get_exchange_rates",
                json!({"base": "CHF", "counter": "GBP"}),
            ),
            (
                "get_historical_rates",
                json!({"base": "CHF", "from": "2024-02-01", "to": "2024-01-01"}),
            ),
            (
                "get_historical_rates",
                json!({"base": "CHF", "from": "2024-01-01", "to": "2024-02-01"}),
            ),
            (
                "get_historical_rates",
                json!({"base": "CHF", "from": "bad", "to": "2024-01-01"}),
            ),
        ] {
            assert_eq!(
                client.call_tool(call(name, args)).await.unwrap().is_error,
                Some(true)
            );
        }
        let http = reqwest::Client::new();
        let unsupported = http.get(&url).send().await.unwrap();
        assert_eq!(
            unsupported.status(),
            reqwest::StatusCode::METHOD_NOT_ALLOWED
        );
        assert_eq!(unsupported.headers()["allow"], "POST");
        let browser_page = http
            .get(&url)
            .header("Accept", "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8")
            .send()
            .await
            .unwrap();
        assert_eq!(browser_page.status(), reqwest::StatusCode::OK);
        assert_eq!(
            browser_page.headers()["content-type"],
            "text/html; charset=utf-8"
        );
        assert_eq!(browser_page.headers()["vary"], "Accept");
        let browser_body = browser_page.text().await.unwrap();
        assert!(browser_body.contains("Exchange Rates MCP"));
        assert!(browser_body.contains("codex mcp add exchange-rates"));
        let protocol_get = http
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .unwrap();
        assert_eq!(
            protocol_get.status(),
            reqwest::StatusCode::METHOD_NOT_ALLOWED
        );
        let oversized = http
            .post(&url)
            .body(vec![b' '; MAX_REQUEST_BYTES + 1])
            .send()
            .await
            .unwrap();
        assert_eq!(oversized.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
        let malformed = http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body("invalid json")
            .send()
            .await
            .unwrap();
        // Preserve the SDK's status for malformed JSON.
        assert_eq!(
            malformed.status(),
            reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
        let browser = reqwest::Client::new()
            .post(url)
            .header("Origin", "https://example.com")
            .send()
            .await
            .unwrap();
        assert_eq!(browser.status(), reqwest::StatusCode::FORBIDDEN);
        client.cancel().await.unwrap();
        let welcome = reqwest::get(format!("http://{address}/")).await.unwrap();
        assert_eq!(welcome.status(), reqwest::StatusCode::OK);
        server_handle.stop(false).await;
    }
}
