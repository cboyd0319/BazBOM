//! Log sanitization to prevent log injection attacks
//!
//! Sanitizes user input before logging to prevent attackers from injecting
//! malicious log entries (e.g., newlines, ANSI escape codes)

/// Sanitize a string for safe logging
///
/// This function removes or escapes characters that could be used for log injection:
/// - Newlines (\n, \r) -> escaped as \\n, \\r
/// - ANSI escape codes -> removed
/// - Null bytes -> removed
/// - Control characters -> removed
///
/// # Examples
///
/// ```
/// use bazbom::security::sanitize_for_log;
///
/// let user_input = "malicious\ninjected log line";
/// let safe = sanitize_for_log(user_input);
/// assert_eq!(safe, "malicious\\ninjected log line");
/// ```
pub fn sanitize_for_log(input: &str) -> String {
    input
        .chars()
        .filter_map(|c| match c {
            // Replace newlines with escaped versions
            '\n' => Some("\\n".to_string()),
            '\r' => Some("\\r".to_string()),
            '\t' => Some("\\t".to_string()),
            // Remove null bytes
            '\0' => None,
            // Remove ANSI escape sequences (ESC character)
            '\x1B' => None,
            // Remove other control characters (0x00-0x1F except tab/newline/carriage return)
            c if c.is_control() && c != '\t' && c != '\n' && c != '\r' => None,
            // Keep printable characters
            c if c.is_ascii_graphic() || c.is_whitespace() || !c.is_ascii() => Some(c.to_string()),
            _ => None,
        })
        .collect()
}

/// Truncate a string to a maximum length for logging
///
/// Prevents log spam from extremely long inputs
pub fn truncate_for_log(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        input.to_string()
    } else {
        format!("{}... (truncated)", &input[..max_len])
    }
}

/// Sanitize and truncate a string for safe logging
///
/// Combines sanitization and truncation in one convenient function
pub fn safe_log(input: &str, max_len: usize) -> String {
    let sanitized = sanitize_for_log(input);
    truncate_for_log(&sanitized, max_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_newlines() {
        let input = "line1\nline2\rline3";
        let output = sanitize_for_log(input);
        assert_eq!(output, "line1\\nline2\\rline3");
    }

    #[test]
    fn test_sanitize_null_bytes() {
        let input = "before\0after";
        let output = sanitize_for_log(input);
        assert_eq!(output, "beforeafter");
    }

    #[test]
    fn test_sanitize_ansi_escape() {
        let input = "\x1B[31mRed text\x1B[0m";
        let output = sanitize_for_log(input);
        assert_eq!(output, "[31mRed text[0m");
    }

    #[test]
    fn test_sanitize_control_chars() {
        let input = "normal\x01\x02\x03text";
        let output = sanitize_for_log(input);
        assert_eq!(output, "normaltext");
    }

    #[test]
    fn test_truncate() {
        let input = "This is a very long string that should be truncated";
        let output = truncate_for_log(input, 20);
        assert_eq!(output, "This is a very long ... (truncated)");
    }

    #[test]
    fn test_safe_log() {
        let input = "malicious\nlog\ninjection\nwith\nvery\nlong\ntext";
        let output = safe_log(input, 30);
        assert!(output.contains("\\n"));
        assert!(output.len() < 50); // Should be truncated
    }

    #[test]
    fn test_log_injection_attack() {
        // Simulate log injection attack attempt
        let malicious_input = "admin logged in\nADMIN LOGGED IN SUCCESSFULLY";
        let safe_output = sanitize_for_log(malicious_input);

        // The output should escape newlines, preventing the injection
        assert_eq!(
            safe_output,
            "admin logged in\\nADMIN LOGGED IN SUCCESSFULLY"
        );

        // When logged, this will appear as one line with visible \n
        // instead of creating a fake admin login log entry
    }
}
