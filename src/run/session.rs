use crate::net::Api;
use crate::state::store;
use crate::support::error::{Error, Result};
use crate::support::log;
use crate::support::private_key;
use crate::wallet::token::{self, Session};
use crate::wallet::Wallet;

pub fn masuk(api: &Api, paksa: bool) -> Result<Session> {
    let hex = private_key().ok_or_else(|| {
        Error::Cfg("fill .env with PRIVATEKEY=0x followed by 64 hex characters".to_string())
    })?;

    let wallet = Wallet::from_hex(&hex)?;

    if !paksa {
        if let Some(cached) = store::read() {
            if cached.wallet.eq_ignore_ascii_case(wallet.address())
                && token::valid(&cached.token, 600)
            {
                api.set_token(Some(cached.token.clone()));
                log::ly("Cached session was reused for this account");
                let session = Session {
                    wallet: cached.wallet,
                    token: cached.token,
                    player: serde_json::Value::Null,
                };
                return Ok(session);
            }
        }
    }

    log::ly(&format!(
        "Wallet {} is signing in",
        log::mask_wallet(wallet.address())
    ));

    let session = token::sign_in(api, &wallet)?;
    api.set_token(Some(session.token.clone()));

    if let Some(exp) = session.expires_at() {
        let _ = store::write(&store::Cached {
            wallet: session.wallet.clone(),
            token: session.token.clone(),
            exp,
        });
    }

    let who = match session.player_id() {
        Some(id) => format!("player {}", id),
        None => "a new player".to_string(),
    };
    log::lg(&format!(
        "Wallet {} signed in as {}",
        log::mask_wallet(&session.wallet),
        who
    ));

    Ok(session)
}
