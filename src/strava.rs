use crate::data_store::DataStore;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use reqwest::{Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub trainer: bool,
    pub gear_id: String,
    pub sport_type: String,
    pub start_date: DateTime<Utc>,
}

lazy_static! {
    static ref BASE_URL: Url = Url::parse("https://www.strava.com/api/v3/").unwrap();
}
pub static TOKEN_URL: &str = "https://www.strava.com/oauth/token";

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenResponse {
    token_type: String,
    expires_at: i64,
    expires_in: i32,
    refresh_token: String,
    access_token: String,
}

pub fn update_and_save_token(token: TokenResponse, data: &mut DataStore) -> Result<()> {
    log::debug!("Updating and saving token -> {:#?}", token);
    data.refresh_token = Some(token.refresh_token);
    data.access_token = Some(token.access_token);
    data.token_expires_at = Some(DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(token.expires_at, 0).unwrap(),
        Utc,
    ));
    data.save()
}

pub async fn refresh_and_save_token(token: &str, data: &mut DataStore) -> Result<()> {
    let client = reqwest::Client::new();
    let params = HashMap::from([
        ("client_id", data.client_id.clone()),
        ("client_secret", data.client_secret.clone()),
        ("grant_type", "refresh_token".into()),
        ("refresh_token", token.into()),
    ]);
    log::debug!("Refreshing tokens");
    if let Ok(response) = client.post(TOKEN_URL).json(&params).send().await {
        if let Ok(json) = response.json::<TokenResponse>().await {
            return update_and_save_token(json, data);
        }
    }
    Ok(())
}

async fn request<T>(
    method: Method,
    path: &str,
    data: &mut DataStore,
    params: Option<HashMap<String, String>>,
    body: Option<HashMap<String, String>>,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let token_expires_at = match data.token_expires_at {
        Some(token_expires_at) => token_expires_at,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Missing token expiration datetime; need to authorize application.",
            ));
        }
    };
    let refresh_token = match data.refresh_token {
        Some(ref refresh_token) => refresh_token.clone(),
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Missing refresh token; need to authorize application.",
            ));
        }
    };
    // add five seconds for clock drift etc
    if Utc::now() + Duration::seconds(5) > token_expires_at {
        refresh_and_save_token(&refresh_token, data).await?;
    }
    let access_token = match data.access_token {
        Some(ref access_token) => access_token.clone(),
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Missing access token; need to authorize application.",
            ));
        }
    };
    let url = BASE_URL
        .join(path)
        .unwrap_or_else(|_| panic!("failed to create URL from path \"{}\"", path));
    log::debug!(
        "Making request to Strava \
        url -> {}
        path -> {}
        method -> {}
        params -> {:#?}
        body -> {:#?}
        token_expires_at -> {}
        refresh_token -> {}
        access_token -> {}",
        url,
        path,
        method,
        params,
        body,
        token_expires_at,
        refresh_token,
        access_token
    );
    let client = reqwest::Client::new();
    let mut req = client
        .request(method.clone(), url)
        .bearer_auth(access_token);
    if let Some(ref body) = body {
        req = req.header("Content-Type", "application/json").json(&body);
    };
    if let Some(ref params) = params {
        req = req.query(&params);
    };
    let res = match req.send().await {
        Ok(res) => match res.error_for_status() {
            Ok(res) => res,
            Err(e) => {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    log::error!("Error 401 from Strava API - probably need to refresh tokens.")
                } else {
                    log::error!("Error from Strava API: {:#?}", e);
                }
                return Err(Error::new(ErrorKind::Other, "Status from Strava API"));
            }
        },
        Err(e) => {
            log::error!("Unknown request error: {:#?}", e);
            return Err(Error::new(ErrorKind::Other, "Error from request to Strava"));
        }
    };
    res.json::<T>()
        .await
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Error converting response to JSON"))
}

pub async fn get_activities_since(
    since: DateTime<Utc>,
    data: &mut DataStore,
) -> Result<Vec<Activity>> {
    request::<Vec<Activity>>(
        Method::GET,
        "athlete/activities",
        data,
        Some(HashMap::from([(
            "after".into(),
            since.timestamp().to_string(),
        )])),
        None,
    )
    .await
}

pub async fn update_activity(
    id: i64,
    body: HashMap<String, String>,
    data: &mut DataStore,
) -> Result<Activity> {
    request::<Activity>(
        Method::PUT,
        &format!("activities/{}", id),
        data,
        None,
        Some(body),
    )
    .await
}
