use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, write},
    io::Result,
};

static FILE_PATH: &str = "data.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct DataStore {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub trainer_bike_id: String,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub last_activity_date: Option<DateTime<Utc>>,
}

impl DataStore {
    pub fn load() -> Result<Self> {
        let json = read_to_string(FILE_PATH)?;
        Ok(serde_json::from_str::<DataStore>(&json)?)
    }

    pub fn save(&mut self) -> Result<()> {
        write(FILE_PATH, serde_json::to_string_pretty(self)?)
    }
}
