use serde_json::{json, Value};

use crate::net::Api;
use crate::support::codec;
use crate::support::error::{Error, Result};
use crate::wallet::{nonce, sign, Wallet};

#[derive(Debug, Clone)]
pub struct Session {
    pub wallet: String,
    pub token: String,
    pub player: Value,
}

impl Session {
    pub fn player_id(&self) -> Option<i64> {
        self.player
            .get("id")
            .and_then(|v| v.as_i64())
            .or_else(|| self.player.get("player").and_then(|v| v.as_i64()))
    }

    pub fn expires_at(&self) -> Option<u64> {
        expiry(&self.token)
    }
}

pub fn sign_in(api: &Api, wallet: &Wallet) -> Result<Session> {
    let message = nonce::message(api, wallet.address())?;
    let signature = sign::personal_sign(wallet.key(), &message);

    let reply = api.wallet_auth(json!({
        "wallet": wallet.address(),
        "signature": signature,
    }))?;

    let token = reply
        .get("token")
        .and_then(|v| v.as_str())
        .or_else(|| reply.get("access_token").and_then(|v| v.as_str()))
        .ok_or_else(|| Error::Auth("the sign in reply carried no token".to_string()))?
        .to_string();

    let player = reply.get("player").cloned().unwrap_or_else(|| Value::Null);

    if token.is_empty() {
        return Err(Error::Auth(
            "the sign in reply carried no token".to_string(),
        ));
    }

    Ok(Session {
        wallet: wallet.address().to_string(),
        token,
        player,
    })
}

pub fn expiry(token: &str) -> Option<u64> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let body = parts.next()?;
    let raw = codec::b64url_decode(body)?;
    let text = String::from_utf8(raw).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    value.get("exp").and_then(|v| v.as_u64())
}

pub fn valid(token: &str, skew_secs: u64) -> bool {
    match expiry(token) {
        Some(exp) => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            exp > now + skew_secs
        }
        None => false,
    }
}

pub fn subject(token: &str) -> Option<String> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let body = parts.next()?;
    let raw = codec::b64url_decode(body)?;
    let text = String::from_utf8(raw).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    value.get("sub").map(|v| {
        v.as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| v.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_body_without_exp_is_not_valid() {
        assert!(!valid("aaa.bbb.ccc", 0));
        assert_eq!(expiry("aaa.bbb.ccc"), None);
    }

    #[test]
    fn an_expired_token_is_not_valid() {
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2MDAwMDAwMDAsInN1YiI6IjQxIn0.sig";
        assert!(!valid(token, 0));
        assert_eq!(expiry(token), Some(1600000000));
        assert_eq!(subject(token).as_deref(), Some("41"));
    }
}
