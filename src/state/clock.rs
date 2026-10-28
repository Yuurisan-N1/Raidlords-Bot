pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn until(target: u64) -> u64 {
    target.saturating_sub(now())
}

pub fn stamp(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let rest = seconds % 60;
    if days > 0 {
        return format!("{:02}:{:02}:{:02}:{:02}", days, hours, minutes, rest);
    }
    format!("{:02}:{:02}:{:02}", hours, minutes, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_keeps_two_digits() {
        assert_eq!(stamp(5), "00:00:05");
        assert_eq!(stamp(3671), "01:01:11");
        assert_eq!(stamp(90061), "01:01:01:01");
    }
}
