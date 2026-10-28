use crate::cli::{probe, selftest, verify};
use crate::net::{proxy, Api};
use crate::run::session::masuk;
use crate::run::{cycle, Mode};
use crate::state::Snapshot;
use crate::support::cfg::{self, Config};
use crate::support::error::{Error, Result};
use crate::support::log;
use crate::treasury;

pub fn dispatch(args: &[String]) -> i32 {
    let command = args.first().map(|s| s.as_str()).unwrap_or("run");

    let outcome = match command {
        "run" => dengan_klien(|api, cfg| cycle::jalankan(api, cfg, Mode::Loop)),
        "once" => dengan_klien(|api, cfg| cycle::jalankan(api, cfg, Mode::Once)),
        "probe" => dengan_klien(|api, _| probe::jalankan(api)),
        "renew" => dengan_klien(|api, _| {
            let session = masuk(api, true)?;
            match session.expires_at() {
                Some(exp) => log::lg(&format!("The new session is valid until {}", exp)),
                None => log::ly("The new session carried no expiry"),
            }
            Ok(())
        }),
        "withdraw" => {
            let amount = args.get(1).and_then(|v| v.parse::<f64>().ok());
            match amount {
                Some(amount) => dengan_klien(|api, _| tarik(api, amount)),
                None => {
                    log::lr("Usage: raidlords-bot withdraw <amount>");
                    return 1;
                }
            }
        }
        "name" => {
            let name = args.get(1).cloned();
            match name {
                Some(name) if !name.trim().is_empty() => dengan_klien(|api, _| {
                    crate::entry::set_name(api, name.trim())?;
                    log::lg(&format!("The village is now named {}", name.trim()));
                    Ok(())
                }),
                _ => {
                    log::lr("Usage: raidlords-bot name <your name>");
                    return 1;
                }
            }
        }
        "entry" => dengan_klien(|api, _| tunjukkan_pintu(api)),
        "paid" => {
            let hash = args.get(1).cloned();
            match hash {
                Some(hash) if hash.trim().len() > 10 => {
                    dengan_klien(|api, _| serahkan_pintu(api, hash.trim()))
                }
                _ => {
                    log::lr("Usage: raidlords-bot paid <transaction hash>");
                    return 1;
                }
            }
        }
        "selftest" => selftest::jalankan(),
        "verify" => verify::jalankan(),
        other => {
            log::lr(&format!("{} is not a command this bot knows", other));
            return 1;
        }
    };

    match outcome {
        Ok(()) => 0,
        Err(err) => {
            log::lr(&format!("The run stopped because {}", log::explain(&err)));
            1
        }
    }
}

fn tunjukkan_pintu(api: &Api) -> Result<()> {
    let gate = crate::entry::Gate::read(&crate::entry::entry::entry(api)?);

    if gate.open() {
        log::lg("The village entry is already paid");
        return Ok(());
    }

    if gate.name.is_none() {
        log::lr("The entry needs a name first so run ./run.sh name <your name>");
        return Ok(());
    }

    let quote = crate::entry::entry::quote(api)?;
    let eth = quote.get("eth").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let wei = quote
        .get("wei")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    let to = quote
        .get("pay_to")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let chain = quote.get("chain_id").and_then(|v| v.as_u64()).unwrap_or(0);

    log::ly(&format!(
        "The entry costs {:.0} usd which is {} eth",
        gate.usd, eth
    ));
    log::ly(&format!("Send it to {} on chain {}", to, chain));
    log::ly(&format!("The amount in wei is {}", wei));
    log::ly("After the transfer run ./run.sh paid <transaction hash>");

    Ok(())
}

fn serahkan_pintu(api: &Api, hash: &str) -> Result<()> {
    let quote = crate::entry::entry::quote(api)?;
    let code = quote
        .get("quote")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    if code.is_null() {
        log::lr("The entry quote was not in the reply so nothing was sent");
        return Ok(());
    }

    crate::entry::entry::submit(api, &code, hash)?;
    log::lg("The entry payment was handed to the server");

    Ok(())
}

fn tarik(api: &Api, amount: f64) -> Result<()> {
    masuk(api, false)?;
    let _ = cycle::tarik_state(api)?;
    treasury::withdraw(api, amount)?;
    log::lg(&format!(
        "{:.2} usd was withdrawn from the treasury",
        amount
    ));
    Ok(())
}

pub fn dengan_klien<F>(work: F) -> Result<()>
where
    F: FnOnce(&Api, &Config) -> Result<()>,
{
    let config = cfg::load()?;
    let proxies = proxy::load();
    let spec = proxies.first();
    if let Some(spec) = spec {
        log::ly(&format!("Using proxy {}", spec.masked()));
    }

    let seed = crate::support::private_key()
        .map(|hex| crate::wallet::Wallet::from_hex(&hex))
        .transpose()?
        .map(|wallet| wallet.address().to_string())
        .unwrap_or_default();

    let api = Api::new(spec, &seed)?;
    masuk(&api, false)?;

    let state = cycle::tarik_state(&api)?;
    ringkas(&state)?;

    work(&api, &config)
}

fn ringkas(state: &Snapshot) -> Result<()> {
    let name = state.name();
    let unnamed = name.is_empty() || name == "unnamed";

    if unnamed {
        log::ly("The account has no name yet");
    }

    let label = if unnamed {
        crate::support::private_key()
            .and_then(|hex| crate::wallet::Wallet::from_hex(&hex).ok())
            .map(|wallet| log::mask_wallet(wallet.address()))
            .unwrap_or_else(|| name.clone())
    } else {
        name
    };

    let clan = match state.clan_tag() {
        Some(tag) => format!("is in clan {}", tag),
        None => "is not in a clan".to_string(),
    };
    log::ly(&format!("Player {} {}", label, clan));

    if state.gold() == 0 && state.army_total() == 0 {
        log::ly("The village is still empty so the first cycle starts from nothing");
    }

    Ok(())
}

pub fn butuh_key() -> Result<()> {
    match crate::support::private_key() {
        Some(_) => Ok(()),
        None => Err(Error::Cfg(
            "fill .env with PRIVATEKEY=0x followed by 64 hex characters".to_string(),
        )),
    }
}
