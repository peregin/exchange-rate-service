[![CircleCI](https://dl.circleci.com/status-badge/img/gh/peregin/exchange-rate-service/tree/master.svg?style=shield)](https://dl.circleci.com/status-badge/redirect/gh/peregin/exchange-rate-service/tree/master)

# Exchange Rate Service
Connects to various data sources on demand and retrieves the latest conversion rates.
It uses a one-hour cache.

Supports the following `json` endpoints:
- /rates/currencies - to retrieve supported currencies
- /rates/:base - to retrieve all FX rates for a given base currency
- /rates/:base/:counter - to retrieve a specific rate for a given currency pair

The root path `/` retrieves a welcome page in `text/html`.

## Requirements
- open source and free usage (non-commercial)
- indicative prices, update frequency is less, but at least once per day
- provide exchange rates from Africa, e.g. UGX
- provide historical data - helps to plot a chart trends for the last 30 days or 3 months

### Currencies to be supported
Currencies supported in the CO-OPERAID online tool and velocorner.com product search. 

| ISO 4217 Currency Code | Currency Name        | Country Code | Country        |
|------------------------|----------------------|--------------|----------------|
| BDT                    | Bangladeshi Taka     | BD           | Bangladesh     |
| CHF                    | Swiss Franc          | CH           | Switzerland    |
| EUR                    | Euro                 | EU           | Eurozone       |
| GBP                    | British Pound        | GB           | United Kingdom |
| KES                    | Kenyan Shilling      | KE           | Kenya          |
| KHR                    | Cambodian Riel       | KH           | Cambodia       |
| LAK                    | Lao Kip              | LA           | Laos           |
| MZN                    | Mozambican Metical   | MZ           | Mozambique     |
| MWK                    | Malawian Kwacha      | MW           | Malawi         |
| UGX                    | Ugandan Shilling     | UG           | Uganda         |
| USD                    | United States Dollar | US           | United States  |


## Data Sources
Data sources and characteristics.

| Site                            | KES, BDT | LAK, KIP | Free | Historical   | Quota      | Source      |
|---------------------------------|----------|----------|------|--------------|------------|-------------|
| ☑️ https://www.frankfurter.app/ | ⛔️       | ⛔️       | ✅    | ✅ timeseries | no         | ECB         |
| https://exchangerate.host       | ✅        | ⛔️       | ✅    | ✅            | 100/mo     | multiple    | 
| https://exchangerate-api.com    | ✅        | ⛔️       | ✅    | ⛔️ paid      | 1500/mo    | 30+         | 
| https://currencyapi.com         | ✅        | ⛔️       | ✅    | ✅            | 300/mo     | multiple    |
| https://openexchangerates.org   | ✅        | ⛔️       | ✅    | ✅ timeseries | 1000/mo    | multiple    |
| https://exchangeratesapi.io     | ✅        | ⛔️       | ✅    | ✅ timeseries | 250/mo     | multiple    |
| https://currency.getgeoapi.com/ | ✅        | ⛔️       | ✅    | ✅            | 100 / day  | multiple    |
| https://rapidapi.com            | ✅        | ⛔️       | ✅    | ⛔️           | 1000 / day | multiple    |
| https://www.abstractapi.com/    | ⛔️       | ⛔️       | ⛔️   | ✅            | ⛔️ 500     | multiple    |
| https://twelvedata.com/         | ✅        | ⛔️       | ✅    | ✅ timeseries | 800 / day  | multiple    |
| https://data.ecb.europa.eu/     | ⛔️       | ⛔️       | ✅    | ⛔️           | ?          | ECB         |
| https://www.centralbank.go.ke/  | ✅        | ⛔️       | ✅    | ✅            | ?          | CBK         |
| https://currencybeacon.com/     | ✅        | ⛔️       | ✅    | ✅ timeseries | 5000 / mo  | multiple    |
| ☑️ fawazahmed0/exchange-api     | ✅        | ⛔️       | ✅    | ✅            | no         | unknown     |
| ☑️ https://www.floatrates.com/  | ✅        | ⛔️       | ✅    | ✅            | no         | CB multiple |

### KES
#### Get indicative rates from Central Bank Kenya
```shell
curl 'https://www.centralbank.go.ke/wp-admin/admin-ajax.php?action=get_wdtable&table_id=193' \
  -H 'accept: application/json, text/javascript, */*; q=0.01' \
  -H 'accept-language: en-US,en;q=0.9,de;q=0.8,hu;q=0.7,ro;q=0.6' \
  -H 'content-type: application/x-www-form-urlencoded; charset=UTF-8' \
  -H 'cookie: _ga=GA1.3.1169526473.1722896765; _gid=GA1.3.1539844224.1723848215; _gat=1; _ga_Y1XLH1GQQ6=GS1.3.1723876245.3.1.1723876411.41.0.0' \
  -H 'origin: https://www.centralbank.go.ke' \
  -H 'priority: u=1, i' \
  -H 'referer: https://www.centralbank.go.ke/forex/' \
  -H 'sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'sec-fetch-dest: empty' \
  -H 'sec-fetch-mode: cors' \
  -H 'sec-fetch-site: same-origin' \
  -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36' \
  -H 'x-requested-with: XMLHttpRequest' \
  --data-raw 'draw=1&columns%5B0%5D%5Bdata%5D=0&columns%5B0%5D%5Bname%5D=date_r&columns%5B0%5D%5Bsearchable%5D=true&columns%5B0%5D%5Borderable%5D=true&columns%5B0%5D%5Bsearch%5D%5Bvalue%5D=&columns%5B0%5D%5Bsearch%5D%5Bregex%5D=false&columns%5B1%5D%5Bdata%5D=1&columns%5B1%5D%5Bname%5D=currency&columns%5B1%5D%5Bsearchable%5D=true&columns%5B1%5D%5Borderable%5D=true&columns%5B1%5D%5Bsearch%5D%5Bvalue%5D=&columns%5B1%5D%5Bsearch%5D%5Bregex%5D=false&columns%5B2%5D%5Bdata%5D=2&columns%5B2%5D%5Bname%5D=ROUND(jx_views_fx_new_rates.mean%2C4)&columns%5B2%5D%5Bsearchable%5D=true&columns%5B2%5D%5Borderable%5D=true&columns%5B2%5D%5Bsearch%5D%5Bvalue%5D=&columns%5B2%5D%5Bsearch%5D%5Bregex%5D=false&order%5B0%5D%5Bcolumn%5D=0&order%5B0%5D%5Bdir%5D=desc&start=0&length=25&search%5Bvalue%5D=&search%5Bregex%5D=false'
```

#### Get KES historical rates from Central Bank Kenya
```shell
curl 'https://www.centralbank.go.ke/wp-admin/admin-ajax.php?action=get_wdtable&table_id=193' \
  -H 'accept: application/json, text/javascript, */*; q=0.01' \
  -H 'accept-language: en-US,en;q=0.9,de;q=0.8,hu;q=0.7,ro;q=0.6' \
  -H 'content-type: application/x-www-form-urlencoded; charset=UTF-8' \
  -H 'origin: https://www.centralbank.go.ke' \
  -H 'priority: u=1, i' \
  -H 'referer: https://www.centralbank.go.ke/forex/' \
  -H 'sec-ch-ua: "Google Chrome";v="125", "Chromium";v="125", "Not.A/Brand";v="24"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'sec-fetch-dest: empty' \
  -H 'sec-fetch-mode: cors' \
  -H 'sec-fetch-site: same-origin' \
  -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36' \
  -H 'x-requested-with: XMLHttpRequest' \
  --data-raw 'draw=5&columns%5B0%5D%5Bdata%5D=0&columns%5B0%5D%5Bname%5D=date_r&columns%5B0%5D%5Bsearchable%5D=true&columns%5B0%5D%5Borderable%5D=true&columns%5B0%5D%5Bsearch%5D%5Bvalue%5D=11%2F03%2F2024~02%2F06%2F2024&columns%5B0%5D%5Bsearch%5D%5Bregex%5D=false&columns%5B1%5D%5Bdata%5D=1&columns%5B1%5D%5Bname%5D=currency&columns%5B1%5D%5Bsearchable%5D=true&columns%5B1%5D%5Borderable%5D=true&columns%5B1%5D%5Bsearch%5D%5Bvalue%5D=S+FRANC&columns%5B1%5D%5Bsearch%5D%5Bregex%5D=false&columns%5B2%5D%5Bdata%5D=2&columns%5B2%5D%5Bname%5D=ROUND(jx_views_fx_new_rates.mean%2C4)&columns%5B2%5D%5Bsearchable%5D=true&columns%5B2%5D%5Borderable%5D=true&columns%5B2%5D%5Bsearch%5D%5Bvalue%5D=&columns%5B2%5D%5Bsearch%5D%5Bregex%5D=false&order%5B0%5D%5Bcolumn%5D=0&order%5B0%5D%5Bdir%5D=desc&start=0&length=100&search%5Bvalue%5D=&search%5Bregex%5D=false&sRangeSeparator=~'
```

### BDT
Get CHF/BDT rates from Central Bank Bangladesh
```shell
curl 'https://www.exchange-rates.org/api/v2/rates/lookup?isoTo=BDT&isoFrom=CHF&amount=1&pageCode=Home' \
-H 'sec-ch-ua: "Google Chrome";v="125", "Chromium";v="125", "Not.A/Brand";v="24"' \
-H 'Referer: https://www.exchange-rates.org/' \
-H 'sec-ch-ua-mobile: ?0' \
-H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36' \
-H 'sec-ch-ua-platform: "macOS"'
```

### Floating Rates
```shell
curl -s https://www.floatrates.com/daily/usd.json | jq .
curl "https://www.floatrates.com/historical-exchange-rates.html?operation=rates&page=historical&currency_date=2024-03-11&base_currency_code=CHF&format_type=xml"
```

### European Central Bank Rates
```shell
curl -s https://api.frankfurter.app/latest | jq .
curl -s https://api.frankfurter.app/currencies | jq .
```

# Docker
```shell
# plain
docker build -t peregin/velocorner.rates .
# buildx
docker buildx build --platform linux/amd64 -t peregin/velocorner.rates:latest --cache-from "type=registry,ref=peregin/velocorner.rates:latest" --cache-to "type=inline" --load .
docker run --rm -it -p 9012:9012 peregin/velocorner.rates
```

# Rust
https://www.rust-lang.org/learn/get-started

Install Rust from shell
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Update `rust-analyzer`, the default is not working with IntelliJ (will cause compilation error when using `cached` library)
```shell
rustup component add rust-analyzer
rustup run stable rust-analyzer --version
```

## Cargo
Useful commands for build and package manager.

```shell
# check for updates and force specific version
cargo update --dry-run
cargo update actix-web --precise 4.5.1
# clean build
cargo clean
# dependency tree
cargo tree
# remove unused dependencies
cargo install cargo-machete && cargo machete
# disable unused features
cargo install cargo-features-manager
cargo features prune
cargo fix
cargo build --release
cargo install --color=always --force cargo-expand
# generate bill of materials
cargo install cargo-sbom
cargo sbom --output-format=cyclone_dx_json_1_4
```

Tips for speeding up builds
https://corrode.dev/blog/tips-for-faster-rust-compile-times/

## Learn Rust
- https://www.rust-lang.org/
- https://github.com/google/comprehensive-rust - great comprehensive course
- https://rust-exercises.com/ - learn by doing, 100 exercises
- https://opensource.googleblog.com/2023/06/rust-fact-vs-fiction-5-insights-from-googles-rust-journey-2022.html
- https://app.pluralsight.com/library/courses/fundamentals-rust/table-of-contents
- https://cheats.rs/
- https://github.com/mre/idiomatic-rust
- https://github.com/rust-unofficial/awesome-rust
- https://github.com/ctjhoa/rust-learning


