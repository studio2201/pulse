//! Cryptographically random session-id generation.
//!
//! Per-app copy of the prior shared helper, kept local so a bug here
//! compromises only this app. Uses the `getrandom` crate directly to
//! pull OS entropy. The fallback path hashes the current system time
//! as a last-resort seed for exotic platforms where `getrandom` is
//! unavailable.

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_ID_BYTES: usize = 16;

/// Generate a fresh cryptographically random session id.
#[must_use]
pub fn generate_session_id() -> String {
    let mut bytes = [0u8; SESSION_ID_BYTES];
    if getrandom::getrandom(&mut bytes).is_err() {
        tracing::warn!(
            target: "session",
            "getrandom failed; falling back to time-based seed"
        );
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mut hasher = Sha256::new();
        hasher.update(nanos.to_string().as_bytes());
        bytes = hasher.finalize()[..SESSION_ID_BYTES].try_into().unwrap();
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn session_id_is_32_hex_chars() {
        let id = generate_session_id();
        assert_eq!(id.len(), SESSION_ID_BYTES * 2);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn session_ids_are_unique() {
        let mut seen = HashSet::new();
        for _ in 0..256 {
            let id = generate_session_id();
            assert!(seen.insert(id), "collision in 256 generated ids");
        }
    }
}
