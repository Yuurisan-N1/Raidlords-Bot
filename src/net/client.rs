use std::sync::RwLock;
use std::time::Duration;

use serde_json::{json, Value};

use crate::net::device::{self, Device};
use crate::net::edge;
use crate::net::pace::Pace;
use crate::net::proxy::ProxySpec;
use crate::support::consts::app::{PACE_JITTER_MS, PACE_MIN_MS};
use crate::support::consts::net::{
    ANON_KEY, BATTLE_FINISH, RPC_PREFIX, SITE_ORIGIN, SITE_REFERER, SUPABASE_URL, TIMEOUT_SECS,
    WALLET_AUTH,
};
use crate::support::error::{Error, Result};

pub struct Api {
    http: reqwest::blocking::Client,
    device: Device,
    pace: Pace,
    token: RwLock<Option<String>>,
}

impl Api {
    pub fn new(proxy: Option<&ProxySpec>, account: &str) -> Result<Api> {
        Api::with_device(proxy, device::for_account(account))
    }

    pub fn with_device(proxy: Option<&ProxySpec>, device: Device) -> Result<Api> {
        let mut builder = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(15))
            .user_agent(device.user_agent.clone())
            .gzip(true)
            .brotli(true);

        if let Some(spec) = proxy {
            builder = spec.apply(builder);
        }

        let http = builder.build().map_err(|e| Error::Http(e.to_string()))?;

        Ok(Api {
            http,
            device,
            pace: Pace::new(PACE_MIN_MS, PACE_JITTER_MS),
            token: RwLock::new(None),
        })
    }

    pub fn set_token(&self, token: Option<String>) {
        if let Ok(mut guard) = self.token.write() {
            *guard = token;
        }
    }

    pub fn token(&self) -> Option<String> {
        self.token.read().ok().and_then(|guard| guard.clone())
    }

    pub fn rpc(&self, name: &str, body: Value) -> Result<Value> {
        let path = format!("{}{}", RPC_PREFIX, name);
        self.post(&path, body, true)
    }

    pub fn function(&self, path: &str, body: Value) -> Result<Value> {
        self.post(path, body, true)
    }

    pub fn wallet_auth(&self, body: Value) -> Result<Value> {
        self.post(WALLET_AUTH, body, false)
    }

    pub fn battle_finish(&self, body: Value) -> Result<Value> {
        self.post(BATTLE_FINISH, body, true)
    }

    pub fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        self.pace.wait();

        let reply = self
            .http
            .get(url)
            .header("accept", "*/*")
            .header("accept-language", self.device.accept_language.as_str())
            .header("cache-control", "no-cache")
            .header("pragma", "no-cache")
            .header("referer", SITE_REFERER)
            .header("sec-ch-ua", self.device.sec_ch_ua.as_str())
            .header("sec-ch-ua-mobile", self.device.sec_ch_ua_mobile.as_str())
            .header(
                "sec-ch-ua-platform",
                self.device.sec_ch_ua_platform.as_str(),
            )
            .send();

        let response = match reply {
            Ok(response) => response,
            Err(err) => {
                return Err(Error::Http(format!(
                    "{} could not be fetched: {}",
                    url, err
                )));
            }
        };

        let status = response.status().as_u16();
        let bytes = match response.bytes() {
            Ok(bytes) => bytes.to_vec(),
            Err(err) => {
                return Err(Error::Http(format!("{} gave no body: {}", url, err)));
            }
        };

        if status >= 400 {
            return Err(Error::Api {
                status,
                message: format!("{} answered {}", url, status),
            });
        }

        Ok(bytes)
    }

    fn request(
        &self,
        url: &str,
        body: &Value,
        with_token: bool,
    ) -> reqwest::blocking::RequestBuilder {
        let mut request = self
            .http
            .post(url)
            .header("accept", "*/*")
            .header("accept-language", self.device.accept_language.as_str())
            .header("apikey", ANON_KEY)
            .header("cache-control", "no-cache")
            .header("content-type", "application/json")
            .header("origin", SITE_ORIGIN)
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("referer", SITE_REFERER)
            .header("sec-ch-ua", self.device.sec_ch_ua.as_str())
            .header("sec-ch-ua-mobile", self.device.sec_ch_ua_mobile.as_str())
            .header(
                "sec-ch-ua-platform",
                self.device.sec_ch_ua_platform.as_str(),
            )
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "cross-site")
            .json(body);

        if with_token {
            if let Some(token) = self.token() {
                request = request.header("authorization", format!("Bearer {}", token));
            }
        }

        request
    }

    fn post(&self, path: &str, body: Value, with_token: bool) -> Result<Value> {
        let url = format!("{}{}", SUPABASE_URL, path);
        let mut attempt = 0;
        loop {
            self.pace.wait();

            let reply = self.request(&url, &body, with_token).send();

            match reply {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let text = response.text().unwrap_or_default();

                    if status == 200 || status == 201 || status == 204 {
                        if text.trim().is_empty() {
                            return Ok(json!({}));
                        }
                        return serde_json::from_str(&text).map_err(|e| {
                            Error::State(format!("the reply could not be read: {}", e))
                        });
                    }

                    let message = server_message(&text);

                    if edge::is_denied(status) {
                        return Err(Error::Denied);
                    }
                    if status == 403 && edge::is_banned(&message) {
                        return Err(Error::Banned);
                    }
                    if edge::transient(status) && attempt < crate::support::consts::app::RETRY_MAX {
                        std::thread::sleep(Duration::from_secs(edge::backoff(attempt)));
                        attempt += 1;
                        continue;
                    }

                    return Err(Error::Api { status, message });
                }
                Err(err) => {
                    if attempt < crate::support::consts::app::RETRY_MAX {
                        std::thread::sleep(Duration::from_secs(edge::backoff(attempt)));
                        attempt += 1;
                        continue;
                    }
                    return Err(Error::Http(err.to_string()));
                }
            }
        }
    }
}

fn server_message(text: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        for key in [
            "message",
            "error_description",
            "msg",
            "error",
            "hint",
            "details",
        ] {
            if let Some(found) = value.get(key).and_then(|v| v.as_str()) {
                if !found.trim().is_empty() {
                    return found.trim().to_string();
                }
            }
        }
    }
    text.trim().chars().take(160).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = "0x415A1b2C3d4E5f60718293a4B5c6D7e8F9012345";

    fn built(with_token: bool) -> reqwest::blocking::Request {
        let path = std::env::temp_dir().join("raidlords-device-headers.json");
        let api = Api::with_device(None, device::for_account_at(&path, ACCOUNT))
            .expect("the client is built");
        api.set_token(Some("session-token".to_string()));
        api.request(
            "https://example.test/rest/v1/rpc/api_state",
            &json!({}),
            with_token,
        )
        .build()
        .expect("the request is built")
    }

    #[test]
    fn every_call_carries_the_headers_a_browser_sends() {
        let request = built(true);

        for name in [
            "accept",
            "accept-language",
            "apikey",
            "content-type",
            "origin",
            "referer",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
        ] {
            assert!(
                request.headers().contains_key(name),
                "the request is missing {}",
                name
            );
        }

        assert_eq!(request.headers()["accept"], "*/*");
        assert_eq!(request.headers()["origin"], SITE_ORIGIN);
        assert_eq!(request.headers()["referer"], SITE_REFERER);
        assert_eq!(request.headers()["sec-fetch-mode"], "cors");
    }

    #[test]
    fn the_call_never_claims_a_client_library_the_browser_does_not_use() {
        let request = built(true);

        assert!(!request.headers().contains_key("accept-profile"));
        assert!(!request.headers().contains_key("x-client-info"));
    }

    #[test]
    fn the_anon_key_rides_along_while_the_bearer_waits_for_sign_in() {
        let anonymous = built(false);
        let signed = built(true);

        assert_eq!(anonymous.headers()["apikey"], ANON_KEY);
        assert!(!anonymous.headers().contains_key("authorization"));
        assert_eq!(signed.headers()["authorization"], "Bearer session-token");
    }

    #[test]
    fn the_device_that_was_cached_is_the_one_that_is_sent() {
        let path = std::env::temp_dir().join("raidlords-device-header.json");
        let _ = std::fs::remove_file(&path);
        let cached = device::for_account_at(&path, ACCOUNT);
        let api = Api::with_device(None, cached.clone()).expect("the client is built");

        assert_eq!(api.device, cached);

        let request = api
            .request(
                "https://example.test/rest/v1/rpc/api_state",
                &json!({}),
                true,
            )
            .build()
            .expect("the request is built");

        assert_eq!(request.headers()["sec-ch-ua"], cached.sec_ch_ua);
        assert_eq!(
            request.headers()["sec-ch-ua-platform"],
            cached.sec_ch_ua_platform
        );
        assert_eq!(request.headers()["accept-language"], cached.accept_language);
    }

    #[test]
    fn a_second_run_keeps_the_same_device_for_the_same_wallet() {
        let path = std::env::temp_dir().join("raidlords-device-second-run.json");
        let _ = std::fs::remove_file(&path);

        let first = Api::with_device(None, device::for_account_at(&path, ACCOUNT))
            .expect("the client is built");
        let again = Api::with_device(None, device::for_account_at(&path, ACCOUNT))
            .expect("the client is built");

        assert_eq!(first.device, again.device);
    }
}
