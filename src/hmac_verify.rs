use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

pub fn verify_hex_sha256(secret: &[u8], body: &[u8], provided: &str) -> bool {
    let sig = provided.trim().trim_start_matches("sha256=");
    let Ok(got) = hex::decode(sig) else {
        return false;
    };
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    let expected = mac.finalize().into_bytes();
    expected.as_slice().ct_eq(&got[..]).into()
}

pub fn verify_shared_secret(expected: &[u8], provided: &[u8]) -> bool {
    if expected.len() != provided.len() {
        return false;
    }
    expected.ct_eq(provided).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_signature_matches() {
        let secret = b"topsecret";
        let body = b"hello world";
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret).unwrap();
        mac.update(body);
        let sig = hex::encode(mac.finalize().into_bytes());
        assert!(verify_hex_sha256(secret, body, &sig));
        assert!(verify_hex_sha256(secret, body, &format!("sha256={sig}")));
    }

    #[test]
    fn wrong_signature_rejected() {
        assert!(!verify_hex_sha256(b"k", b"body", "deadbeef"));
        assert!(!verify_hex_sha256(b"k", b"body", "not-hex"));
        assert!(!verify_hex_sha256(b"k", b"body", ""));
    }

    #[test]
    fn shared_secret_check() {
        assert!(verify_shared_secret(b"abc", b"abc"));
        assert!(!verify_shared_secret(b"abc", b"abcd"));
        assert!(!verify_shared_secret(b"abc", b"abd"));
    }
}
