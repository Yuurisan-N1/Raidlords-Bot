use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::support::consts::app::DEVICE_FILE;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub user_agent: String,
    pub sec_ch_ua: String,
    pub sec_ch_ua_mobile: String,
    pub sec_ch_ua_platform: String,
    pub accept_language: String,
}

const WINDOWS: [(&str, &str); 6] = [
    ("Windows NT 10.0; Win64; x64", "Windows"),
    ("Windows NT 10.0; Win64; x64", "Windows"),
    ("Windows NT 10.0; Win64; x64", "Windows"),
    ("Windows NT 10.0; WOW64", "Windows"),
    ("Windows NT 11.0; Win64; x64", "Windows"),
    ("Windows NT 10.0; Win64; x64", "Windows"),
];

const ANDROID: [(&str, &str, &str); 6] = [
    ("Android 14", "SM-S918B", "?1"),
    ("Android 14", "Pixel 8", "?1"),
    ("Android 13", "SM-A546E", "?1"),
    ("Android 13", "23021RAAEG", "?1"),
    ("Android 12", "CPH2451", "?1"),
    ("Android 14", "RMX3630", "?1"),
];

const LANGUAGES: [&str; 5] = [
    "en-US,en;q=0.9",
    "en-US,en;q=0.9,id-ID;q=0.8,id;q=0.7",
    "id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7",
    "en-GB,en;q=0.9,en-US;q=0.8",
    "en-US,en;q=0.9,ja-JP;q=0.8,ja;q=0.7",
];

fn draw() -> u64 {
    let mut raw = [0u8; 8];

    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        if file.read_exact(&mut raw).is_ok() {
            return u64::from_le_bytes(raw);
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let mixed = nanos ^ ((std::process::id() as u64) << 32);
    mixed.wrapping_mul(0x9e3779b97f4a7c15)
}

fn pick(pool: u64, span: usize) -> usize {
    (pool % span as u64) as usize
}

pub fn generate() -> Device {
    let pool = draw();
    let version = 152 + pick(pool >> 8, 7);
    let language = LANGUAGES[pick(pool >> 16, LANGUAGES.len())].to_string();
    let desktop = pick(pool >> 24, 10) >= 3;

    if desktop {
        let (os, platform) = WINDOWS[pick(pool >> 32, WINDOWS.len())];
        let brand = pick(pool >> 40, 3);

        let user_agent = format!(
            "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
            os, version
        );

        let sec_ch_ua = match brand {
            0 => format!(
                "\"Chromium\";v=\"{0}\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"{0}\"",
                version
            ),
            1 => format!(
                "\"Not:A-Brand\";v=\"24\", \"Chromium\";v=\"{0}\", \"Google Chrome\";v=\"{0}\"",
                version
            ),
            _ => format!(
                "\"Google Chrome\";v=\"{0}\", \"Chromium\";v=\"{0}\", \"Not:A-Brand\";v=\"24\"",
                version
            ),
        };

        return Device {
            user_agent,
            sec_ch_ua,
            sec_ch_ua_mobile: "?0".to_string(),
            sec_ch_ua_platform: format!("\"{}\"", platform),
            accept_language: language,
        };
    }

    let (android, model, mobile) = ANDROID[pick(pool >> 32, ANDROID.len())];

    Device {
        user_agent: format!(
            "Mozilla/5.0 (Linux; {}; {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Mobile Safari/537.36",
            android, model, version
        ),
        sec_ch_ua: format!(
            "\"Chromium\";v=\"{0}\", \"Not:A-Brand\";v=\"24\", \"Google Chrome\";v=\"{0}\"",
            version
        ),
        sec_ch_ua_mobile: mobile.to_string(),
        sec_ch_ua_platform: "\"Android\"".to_string(),
        accept_language: language,
    }
}

fn load(path: &Path) -> BTreeMap<String, Device> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => return BTreeMap::new(),
    };

    serde_json::from_str(&text).unwrap_or_default()
}

fn save(path: &Path, devices: &BTreeMap<String, Device>) {
    if let Ok(text) = serde_json::to_string_pretty(devices) {
        let _ = std::fs::write(path, text + "\n");
    }
}

fn key_of(address: &str) -> String {
    let key = address.trim().to_ascii_lowercase();

    if key.is_empty() {
        "unbound".to_string()
    } else {
        key
    }
}

pub fn for_account(address: &str) -> Device {
    for_account_at(Path::new(DEVICE_FILE), address)
}

fn for_account_at(path: &Path, address: &str) -> Device {
    let mut devices = load(path);
    let key = key_of(address);

    if let Some(found) = devices.get(&key) {
        return found.clone();
    }

    let device = generate();
    devices.insert(key, device.clone());
    save(path, &devices);

    device
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("rl-device-{}-{}.json", name, draw()));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn a_device_never_looks_like_linux_or_macos() {
        for _ in 0..300 {
            let device = generate();

            assert!(!device.user_agent.contains("Macintosh"));
            assert!(!device.user_agent.contains("X11"));
            assert!(!device.sec_ch_ua_platform.contains("macOS"));
            assert!(!device.sec_ch_ua_platform.contains("Linux"));

            let desktop = device.user_agent.contains("Windows NT");
            let android = device.user_agent.contains("Android");

            assert!(desktop || android, "{}", device.user_agent);
        }
    }

    #[test]
    fn the_platform_hint_matches_the_user_agent() {
        for _ in 0..300 {
            let device = generate();

            if device.user_agent.contains("Windows NT") {
                assert_eq!(device.sec_ch_ua_platform, "\"Windows\"");
                assert_eq!(device.sec_ch_ua_mobile, "?0");
            } else {
                assert_eq!(device.sec_ch_ua_platform, "\"Android\"");
                assert_eq!(device.sec_ch_ua_mobile, "?1");
            }
        }
    }

    #[test]
    fn the_brand_hint_carries_the_same_version_as_the_user_agent() {
        for _ in 0..300 {
            let device = generate();
            let version = device
                .user_agent
                .split("Chrome/")
                .nth(1)
                .and_then(|rest| rest.split('.').next())
                .expect("the user agent names a version");

            assert!(
                device.sec_ch_ua.contains(&format!("v=\"{}\"", version)),
                "{} vs {}",
                device.sec_ch_ua,
                device.user_agent
            );
        }
    }

    #[test]
    fn a_generated_device_is_not_the_same_for_every_account() {
        let mut seen = std::collections::HashSet::new();

        for _ in 0..40 {
            seen.insert(generate().user_agent);
        }

        assert!(seen.len() > 1);
    }

    #[test]
    fn one_account_keeps_the_same_device_across_runs() {
        let path = scratch("stable");
        let first = for_account_at(&path, "0xAbC0000000000000000000000000000000000001");
        let again = for_account_at(&path, "0xabc0000000000000000000000000000000000001");

        assert_eq!(first, again);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_second_wallet_is_added_without_dropping_the_first() {
        let path = scratch("append");
        let old = for_account_at(&path, "0x1111111111111111111111111111111111111111");
        let fresh = for_account_at(&path, "0x2222222222222222222222222222222222222222");

        assert_ne!(old, fresh);

        let text = std::fs::read_to_string(&path).expect("the file is written");
        let stored: BTreeMap<String, Device> =
            serde_json::from_str(&text).expect("the file parses");

        assert_eq!(stored.len(), 2);
        assert_eq!(
            stored.get("0x1111111111111111111111111111111111111111"),
            Some(&old)
        );
        assert_eq!(
            stored.get("0x2222222222222222222222222222222222222222"),
            Some(&fresh)
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_file_is_written_pretty_with_one_key_per_wallet() {
        let path = scratch("pretty");
        for_account_at(&path, "0x3333333333333333333333333333333333333333");

        let text = std::fs::read_to_string(&path).expect("the file is written");

        assert!(text.contains("\n  \"0x3333333333333333333333333333333333333333\": {"));
        assert!(text.lines().count() > 5);

        let _ = std::fs::remove_file(&path);
    }
}
