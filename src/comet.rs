use super::plurk::{Plurk, PlurkError};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_qs as qs;
use std::time::Duration;

pub struct PlurkComet {
    base_url: String,
    channel: String,
    offset: i64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct CometDatas {
    channel: String,
    offset: i64,
}

impl PlurkComet {
    pub fn new(comet_url: &str) -> Result<PlurkComet, PlurkError> {
        let url = Url::parse(comet_url).unwrap();
        let query = url.query().unwrap();

        let comet_datas: CometDatas = match qs::from_str(query) {
            Ok(t) => t,
            Err(e) => return Err(PlurkError::new(1, e.to_string())),
        };

        let url = url.join("comet").unwrap();

        Ok(PlurkComet {
            base_url: Url::to_string(&url),
            channel: comet_datas.channel,
            offset: comet_datas.offset,
        })
    }
    pub fn print(&self) {
        println!(
            "<Comet>:\n\tBase Url: {}\n\tChannel: {}\n\tOffset: {}",
            self.base_url, self.channel, self.offset
        );
    }
    pub fn as_str(&self) {
        let url = Url::parse_with_params(
            &self.base_url,
            &[
                ("channel", &self.channel),
                ("offset", &self.offset.to_string()),
            ],
        )
        .unwrap().as_str();
    }

    pub fn call_once_mut(&mut self) -> Result<Value, PlurkError> {
        let url = Url::parse_with_params(
            &self.base_url,
            &[
                ("channel", &self.channel),
                ("offset", &self.offset.to_string()),
            ],
        )
        .unwrap();

        let client = Client::new();

        let res = match client.get(url).timeout(Duration::from_secs(60)).send() {
            Ok(t) => t,
            Err(e) => return Err(PlurkError::new(1, e.to_string())),
        };

        let text = match res.text() {
            Ok(t) => t,
            Err(e) => return Err(PlurkError::new(1, e.to_string())),
        };

        let res = PlurkComet::query(&text)?;
        // Ok(res)
        self.offset = res["new_offset"].as_i64().unwrap();
        Ok(res["data"].clone())
    }
    pub fn update_offset(self, offset: i64) -> PlurkComet {
        PlurkComet {
            base_url: self.base_url,
            channel: self.channel,
            offset: offset,
        }
    }
    fn query(comet_callback: &str) -> Result<Value, PlurkError> {
        let re = Regex::new(r"CometChannel.scriptCallback\((.*)\);").unwrap();
        let mat = re.captures(comet_callback).unwrap();
        match serde_json::from_str(&mat[1]) {
            Ok(t) => Ok(t),
            Err(e) => return Err(PlurkError::new(1, e.to_string())),
        }
    }
}
