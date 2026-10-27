use reqwest::Client;
use serde::de::DeserializeOwned;
use std::{sync::LazyLock, time::Duration};

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to initialize HTTP client")
});

pub(super) async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, reqwest::Error> {
    CLIENT
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{web, App, HttpResponse, HttpServer};
    use serde_json::{json, Value};
    use std::net::TcpListener;

    #[actix_web::test]
    async fn json_requests_handle_success_status_errors_and_invalid_json() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = HttpServer::new(|| {
            App::new()
                .route(
                    "/success",
                    web::get().to(|| async { HttpResponse::Ok().json(json!({"rate": 1.25})) }),
                )
                .route(
                    "/unavailable",
                    web::get().to(|| async {
                        HttpResponse::ServiceUnavailable().json(json!({"error": "unavailable"}))
                    }),
                )
                .route(
                    "/invalid",
                    web::get().to(|| async { HttpResponse::Ok().body("invalid JSON") }),
                )
        })
        .workers(1)
        .listen(listener)
        .unwrap()
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);

        let success = get_json::<Value>(&format!("{url}/success")).await;
        let unavailable = get_json::<Value>(&format!("{url}/unavailable")).await;
        let invalid = get_json::<Value>(&format!("{url}/invalid")).await;
        handle.stop(false).await;

        assert_eq!(success.unwrap(), json!({"rate": 1.25}));
        assert_eq!(
            unavailable.unwrap_err().status(),
            Some(reqwest::StatusCode::SERVICE_UNAVAILABLE)
        );
        assert!(invalid.unwrap_err().is_decode());
    }
}
