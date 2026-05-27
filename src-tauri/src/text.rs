//! UTF-8 safe text helpers.

/// Return at most `max_chars` Unicode scalar values.
pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

/// Truncate by byte budget without cutting through a UTF-8 character.
pub fn truncate_utf8_bytes(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }

    let mut end = max_bytes.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Return at most `max_chars` characters and append `suffix` only if truncated.
pub fn truncate_chars_with_suffix(text: &str, max_chars: usize, suffix: &str) -> String {
    let truncated = truncate_chars(text, max_chars);
    if truncated.len() < text.len() {
        format!("{truncated}{suffix}")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_handles_cjk_and_emoji() {
        let text = "预".repeat(250) + "✨";
        let truncated = truncate_chars(&text, 200);
        assert_eq!(truncated.chars().count(), 200);
        assert!(truncated.is_char_boundary(truncated.len()));
    }

    #[test]
    fn truncate_utf8_bytes_backs_up_to_boundary() {
        let text = "a".repeat(499) + "预报";
        let truncated = truncate_utf8_bytes(&text, 500);
        assert_eq!(truncated.len(), 499);
        assert!(truncated.ends_with('a'));
    }

    #[test]
    fn truncate_chars_with_suffix_only_adds_suffix_when_needed() {
        assert_eq!(truncate_chars_with_suffix("hello", 10, "..."), "hello");
        assert_eq!(truncate_chars_with_suffix("hello", 3, "..."), "hel...");
    }
}
