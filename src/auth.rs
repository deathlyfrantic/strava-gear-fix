use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log::LevelFilter;
use reqwest::Url;
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    process::{exit, Command},
};
use strava_gear_fix::{
    data_store::DataStore,
    strava::{update_and_save_token, TokenResponse, TOKEN_URL},
};

static AUTHORIZE_URL: &str = "https://www.strava.com/oauth/authorize";

async fn get_token(code: &str, data: &DataStore) -> Result<reqwest::Response, reqwest::Error> {
    log::debug!("Getting authorization tokens");
    let client = reqwest::Client::new();
    let params = HashMap::from([
        ("client_id", data.client_id.as_str()),
        ("client_secret", data.client_secret.as_str()),
        ("code", code),
        ("grant_type", "authorization_code"),
    ]);
    client.post(TOKEN_URL).json(&params).send().await
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut data = DataStore::load().expect("failed to load data");
    let params: HashMap<String, String> = req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            return Ok(Response::builder()
                .status(500)
                .body(
                    params
                        .get("error")
                        .unwrap_or(&"unknown error".into())
                        .to_string()
                        .into(),
                )
                .unwrap());
        }
    };
    if req.uri().path() == "/strava-auth" {
        match get_token(code, &data).await {
            Ok(response) => match response.json::<TokenResponse>().await {
                Ok(json) => match update_and_save_token(json, &mut data) {
                    Ok(()) => {
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            exit(0);
                        });
                        return Ok(Response::new(Body::from("OK")));
                    }
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(500)
                            .body(e.to_string().into())
                            .unwrap());
                    }
                },
                Err(e) => {
                    return Ok(Response::builder()
                        .status(500)
                        .body(e.to_string().into())
                        .unwrap());
                }
            },
            Err(e) => {
                return Ok(Response::builder()
                    .status(500)
                    .body(e.to_string().into())
                    .unwrap());
            }
        };
    }
    Ok(Response::builder()
        .status(404)
        .body("Not Found".into())
        .unwrap())
}

fn open_auth_url_in_browser(data: &DataStore) {
    let url = Url::parse_with_params(
        AUTHORIZE_URL,
        &[
            ("client_id", data.client_id.as_str()),
            ("redirect_uri", "http://localhost:8000/strava-auth"),
            ("response_type", "code"),
            ("scope", "activity:read_all,activity:write"),
        ],
    )
    .expect("failed to parse Oauth URL");
    log::debug!("Opening authorization URL \"{}\"", url.as_str());
    Command::new("open")
        .arg(url.as_str())
        .spawn()
        .expect("failed to open Strava Oauth URL");
}

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()
        .expect("Failed to initiate logger");
    let data = DataStore::load().expect("failed to load data");
    // open auth url
    open_auth_url_in_browser(&data);
    // start server to do token exchange
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler)) });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
