/// Escape a filesystem path for use as a systemd unit instance name.
///
/// Implements the same algorithm as `systemd-escape --path`:
/// - The path is cleaned (leading/trailing slashes stripped, collapsed)
/// - `/` becomes `-`
/// - Any character outside `[a-zA-Z0-9:_.]` is hex-escaped as `\xHH`
/// - The root path `/` becomes `-`
///
/// See systemd.unit(5) for the full specification.
pub fn escape_path(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return "-".to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch == '/' {
            out.push('-');
        } else if ch.is_ascii_alphanumeric() || ch == ':' || ch == '_' || ch == '.' {
            out.push(ch);
        } else {
            write!(out, "\\x{:02x}", ch as u32).unwrap();
        }
    }
    out
}

use std::fmt::Write;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_simple_path() {
        assert_eq!(escape_path("/home/user/project"), "home-user-project");
    }

    #[test]
    fn test_escape_root() {
        assert_eq!(escape_path("/"), "-");
    }

    #[test]
    fn test_escape_trailing_slashes() {
        assert_eq!(escape_path("/home/user/"), "home-user");
    }

    #[test]
    fn test_escape_special_chars() {
        assert_eq!(
            escape_path("/home/user/my project"),
            "home-user-my\\x20project"
        );
    }
}
