use oauth::serializer::auth::{self, HmacSha1Authorizer};
use oauth::serializer::{Serializer, SerializerExt};
use reqwest::blocking::{multipart, Client};
use reqwest::Url;
use serde::{Deserialize, Serialize};
pub use serde_json::Value;
use serde_qs as qs;
use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::io::{self, stdout, BufRead, Write};

const REQUEST_TOKEN_URL: &str = "https://www.plurk.com/OAuth/request_token";
const AUTHORIZE_URL: &str = "https://www.plurk.com/OAuth/authorize";
const ACCESS_TOKEN_URL: &str = "https://www.plurk.com/OAuth/access_token";
const BASE_URL: &str = "https://www.plurk.com";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct RequestToken {
    oauth_token: String,
    oauth_token_secret: String,
}

pub struct Plurk {
    client: String,
    client_secret: String,
    token: String,
    token_secret: String,
    authed: bool,
    rc: Client,
}

#[derive(Serialize, Deserialize)]
pub struct PlurkError {
    code: usize,
    message: String,
}

impl PlurkError {
    pub fn new(code: usize, message: String) -> PlurkError {
        PlurkError {
            code: code,
            message: message
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TokValues {
    key: String,
    secret: String,
}
#[derive(Serialize, Deserialize)]
struct Keys {
    client: TokValues,
    token: TokValues,
}

impl fmt::Display for PlurkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.code {
            404 => "Sorry, Can not find the Page!",
            _ => "Sorry, something is wrong! Please Try Again!",
        };

        write!(f, "{}", err_msg)
    }
}

impl fmt::Debug for PlurkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PlurkError {{ code: {}, message: {} }}",
            self.code, self.message
        )
    }
}

impl Plurk {
    pub fn new(c: &str, cs: &str, t: Option<String>, ts: Option<String>) -> Plurk {
        match (t, ts) {
            (Some(_t), Some(_ts)) => Plurk {
                client: c.to_string(),
                client_secret: cs.to_string(),
                token: _t,
                token_secret: _ts,
                authed: true,
                rc: Client::new(),
            },
            (_, _) => Plurk {
                client: c.to_string(),
                client_secret: cs.to_string(),
                token: "".to_string(),
                token_secret: "".to_string(),
                authed: false,
                rc: Client::new(),
            },
        }
    }
    pub fn from_file(file: &str) -> Result<Plurk, PlurkError> {
        let mut file = match fs::File::open(&file) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };
        let keys: Keys = match toml::from_str(contents.as_str()) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };
        let authed: bool = keys.token.key.is_empty() || keys.token.secret.is_empty();

        Ok(Plurk {
            client: keys.client.key,
            client_secret: keys.client.secret,
            token: keys.token.key,
            token_secret: keys.token.secret,
            authed: !authed,
            rc: Client::new(),
        })
    }
    pub fn is_authed(&self) -> bool {
        self.authed
    }
    pub fn auth(self) -> Result<Plurk, PlurkError> {
        if self.is_authed() {
            return Ok(self);
        }
        let plurk = self.request_token()?;
        let auth_url = plurk.get_auth_url();
        println!("Please access the auth url: {}", auth_url);

        print!("Input verifier: ");
        let _ = stdout().flush();
        let mut verifier = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut verifier).unwrap();
        let verifier = verifier.trim_end();

        let plurk = plurk.get_access(verifier.to_string())?;
        Ok(plurk)
    }
    pub fn print(&self) {
        println!(
            "<Plurk>:\n\tClient: {}\n\tClient Token: {}\n\tToken: {}\n\tToken Secret: {}",
            self.client, self.client_secret, self.token, self.token_secret
        );
    }
    pub fn write_in_file(&self, file: &str) -> Result<(), PlurkError> {
        let keys = Keys {
            client: TokValues {
                key: self.client.to_owned(),
                secret: self.client_secret.to_owned(),
            },
            token: TokValues {
                key: self.token.to_owned(),
                secret: self.token_secret.to_owned(),
            },
        };
        let toml = match toml::to_string(&keys) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };
        match fs::write(file, toml) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };
        Ok(())
    }
    pub fn request_token(self) -> Result<Plurk, PlurkError> {
        let token = oauth::Token::from_parts(
            &self.client,
            &self.client_secret,
            &self.token,
            &self.token_secret,
        );
        let authorization_header = oauth::post(REQUEST_TOKEN_URL, &(), &token, oauth::HmacSha1);
        // let client = Client::new();
        let res = match self
            .rc
            .post(REQUEST_TOKEN_URL)
            .header("Authorization", authorization_header)
            .send()
        {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        let text = match res.text() {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        let tokens: RequestToken = match qs::from_str(&text) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        Ok(Plurk {
            client: self.client,
            client_secret: self.client_secret,
            token: tokens.oauth_token,
            token_secret: tokens.oauth_token_secret,
            authed: false,
            rc: self.rc,
        })
    }

    pub fn get_auth_url(&self) -> String {
        return format!(
            "{authorization_url}?oauth_token={token}",
            authorization_url = AUTHORIZE_URL,
            token = self.token
        );
    }

    pub fn get_access(self, verifier: String) -> Result<Plurk, PlurkError> {
        let token = oauth::Token::from_parts(
            &self.client,
            &self.client_secret,
            &self.token,
            &self.token_secret,
        );

        let authorization_header = oauth::Builder::with_token(token, oauth::HmacSha1)
            .verifier(verifier.as_str())
            .post(ACCESS_TOKEN_URL, &());

        let res = match self
            .rc
            .post(ACCESS_TOKEN_URL)
            .header("Authorization", authorization_header)
            .send()
        {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        let text = match res.text() {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        let tokens: RequestToken = match qs::from_str(&text) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        Ok(Plurk {
            client: self.client,
            client_secret: self.client_secret,
            token: tokens.oauth_token,
            token_secret: tokens.oauth_token_secret,
            authed: true,
            rc: self.rc,
        })
    }

    pub fn request(
        &self,
        api: &str,
        data: Option<BTreeSet<(&str, &str)>>,
        file: Option<BTreeSet<(&str, &str)>>,
    ) -> Result<Value, PlurkError> {
        // if self.authed == false {
        //     return Err(PlurkError{
        //         code: 1,
        //         message: "Oauth not authed".to_string()
        //     });
        // }
        let token = oauth::Token::from_parts(
            &self.client,
            &self.client_secret,
            &self.token,
            &self.token_secret,
        );

        let base = match Url::parse(BASE_URL) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 4,
                    message: e.to_string(),
                })
            }
        };
        let url = match base.join(api) {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 5,
                    message: e.to_string(),
                })
            }
        };

        let mut form = multipart::Form::new();

        if let Some(f) = &file {
            for (name, value) in f {
                form = match form.file(name.to_string(), value.to_string()) {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(PlurkError {
                            code: 1,
                            message: e.to_string(),
                        })
                    }
                };
            }
        }
        let authorization_header = match &data {
            Some(d) => oauth::post(&url, &d.clone(), &token, oauth::HmacSha1),
            _ => oauth::post(&url, &(), &token, oauth::HmacSha1),
        };

        let pre_send = self
            .rc
            .post(url)
            .header("Authorization", authorization_header)
            .form(&data)
            .multipart(form);

        let res = match pre_send.send() {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 1,
                    message: e.to_string(),
                })
            }
        };

        let text = match res.text() {
            Ok(t) => t,
            Err(e) => {
                return Err(PlurkError {
                    code: 2,
                    message: e.to_string(),
                })
            }
        };

        match serde_json::from_str(&text) {
            Ok(t) => Ok(t),
            Err(e) => {
                return Err(PlurkError {
                    code: 3,
                    message: e.to_string(),
                })
            }
        }
    }
}

pub fn print_user(user: Value) {
    println!("{}", "=".repeat(40));
    println!("Display name: {}", user["display_name"]);
    println!("Is channel:   {}", user["is_channel"]);
    println!("Nick name:    {}", user["nick_name"]);
    println!("Has Prof Img: {}", user["has_profile_image"]);
    println!("Location:     {}", user["location"]);
    println!("Birth:        {}", user["date_of_birth"]);
    println!("Relationship: {}", user["relationship"]);
    println!("Avatar:       {}", user["avatar"]);
    println!("Full name:    {}", user["full_name"]);
    println!("Gender:       {}", user["gender"]);
    println!("Recruited:    {}", user["recruited"]);
    println!("Id:           {}", user["id"]);
    println!("Karma:        {}", user["karma"]);
    println!("{}", "=".repeat(40));
}
