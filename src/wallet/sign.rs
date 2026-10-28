use k256::ecdsa::SigningKey;

use crate::support::codec;
use crate::wallet::address::keccak;

pub fn personal_digest(message: &str) -> [u8; 32] {
    let prefix = format!("\u{19}Ethereum Signed Message:\n{}", message.len());
    let mut payload = Vec::with_capacity(prefix.len() + message.len());
    payload.extend_from_slice(prefix.as_bytes());
    payload.extend_from_slice(message.as_bytes());
    keccak(&payload)
}

pub fn personal_sign(key: &SigningKey, message: &str) -> String {
    let digest = personal_digest(message);
    let (signature, recovery) = key
        .sign_prehash_recoverable(&digest)
        .expect("signing a 32 byte digest cannot fail");
    let mut out = Vec::with_capacity(65);
    out.extend_from_slice(&signature.to_bytes());
    out.push(recovery.to_byte() + 27);
    format!("0x{}", codec::hex_encode(&out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    #[test]
    fn a_signature_recovers_the_signer() {
        let key = SigningKey::from_slice(&[0x46u8; 32]).unwrap();
        let message = "beta.raidlords.com wants you to sign in with your wallet";
        let signed = personal_sign(&key, message);
        let raw = codec::hex_decode(&signed).unwrap();
        assert_eq!(raw.len(), 65);

        let v = raw[64];
        assert!(v == 27 || v == 28);
        let recovery = RecoveryId::from_byte(v - 27).unwrap();
        let signature = Signature::from_slice(&raw[..64]).unwrap();
        let recovered =
            VerifyingKey::recover_from_prehash(&personal_digest(message), &signature, recovery)
                .unwrap();
        assert_eq!(recovered, *key.verifying_key());
    }

    #[test]
    fn the_prefix_carries_the_message_length() {
        let digest = personal_digest("abc");
        let manual = keccak(b"\x19Ethereum Signed Message:\n3abc");
        assert_eq!(digest, manual);
    }
}
