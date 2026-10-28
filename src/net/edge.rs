use crate::support::consts::app::RECONNECT_SECS;

pub fn transient(status: u16) -> bool {
    status == 429 || status == 408 || status >= 500
}

pub fn backoff(attempt: u32) -> u64 {
    let shift = attempt.min(4);
    (RECONNECT_SECS << shift).min(30)
}

pub fn is_denied(status: u16) -> bool {
    status == 401
}

pub fn is_banned(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("banned") || lower.contains("suspended")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_busy_server_is_retried() {
        assert!(transient(429));
        assert!(transient(503));
        assert!(!transient(400));
        assert!(!transient(401));
    }

    #[test]
    fn the_backoff_stops_at_thirty_seconds() {
        assert_eq!(backoff(0), RECONNECT_SECS);
        assert_eq!(backoff(1), RECONNECT_SECS * 2);
        assert_eq!(backoff(9), 30);
    }
}
