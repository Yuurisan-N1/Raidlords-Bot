use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};

use crate::support::codec;
use crate::support::error::{Error, Result};

pub fn keccak(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&out);
    hash
}

pub struct Wallet {
    key: SigningKey,
    address: String,
}

impl Wallet {
    pub fn from_hex(hex: &str) -> Result<Wallet> {
        let raw = codec::hex_decode(hex)
            .ok_or_else(|| Error::Wallet("the private key is not readable".to_string()))?;
        if raw.len() != 32 {
            return Err(Error::Wallet(
                "the private key is not 32 bytes long".to_string(),
            ));
        }
        let key = SigningKey::from_slice(&raw)
            .map_err(|_| Error::Wallet("the private key is not valid".to_string()))?;

        let public = key.verifying_key().to_encoded_point(false);
        let hash = keccak(&public.as_bytes()[1..]);
        let address = checksum(&codec::hex_encode(&hash[12..]));

        Ok(Wallet { key, address })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn key(&self) -> &SigningKey {
        &self.key
    }
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .finish()
    }
}

pub fn checksum(lower_hex: &str) -> String {
    let lower = lower_hex.trim_start_matches("0x").to_ascii_lowercase();
    let hash = keccak(lower.as_bytes());
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (idx, ch) in lower.chars().enumerate() {
        if ch.is_ascii_digit() {
            out.push(ch);
            continue;
        }
        let nibble = (hash[idx / 2] >> if idx % 2 == 0 { 4 } else { 0 }) & 0x0f;
        if nibble >= 8 {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_key_gives_the_known_address() {
        let wallet =
            Wallet::from_hex("4646464646464646464646464646464646464646464646464646464646464646")
                .unwrap();
        assert_eq!(
            wallet.address(),
            "0x9d8A62f656a8d1615C1294fd71e9CFb3E4855A4F"
        );
    }

    #[test]
    fn a_short_key_is_refused() {
        assert!(Wallet::from_hex("0x1234").is_err());
    }

    #[test]
    fn the_checksum_round_trips() {
        assert_eq!(
            checksum("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed"),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );
    }
}
