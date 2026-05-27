//! Token Compressor — reduces token count of tool output before LLM sees it.
//!
//! Inspired by OpenHuman's TokenJuice: HTML to Markdown, URL shortening,
//! whitespace normalization, and smart truncation.

/// Compress tool output to reduce token usage
pub struct TokenCompressor {
    max_chars: usize,
}

impl TokenCompressor {
    /// Create a new compressor with a character limit
    pub fn new(max_chars: usize) -> Self {
        Self { max_chars }
    }

    /// Compress HTML content to plain text
    pub fn compress_html(&self, html: &str) -> String {
        let mut result = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut in_style = false;
        let mut in_script = false;
        let mut tag_name = String::new();

        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '<' => {
                    in_tag = true;
                    tag_name.clear();
                }
                '>' => {
                    in_tag = false;
                    let name_lower = tag_name.to_lowercase();
                    if name_lower == "script" {
                        in_script = true;
                    } else if name_lower == "/script" {
                        in_script = false;
                    } else if name_lower == "style" {
                        in_style = true;
                    } else if name_lower == "/style" {
                        in_style = false;
                    }
                    tag_name.clear();
                }
                _ => {
                    if in_tag {
                        if chars[i].is_alphabetic() || chars[i] == '/' {
                            tag_name.push(chars[i]);
                        }
                    } else if !in_script && !in_style {
                        result.push(chars[i]);
                    }
                }
            }
            i += 1;
        }

        // Normalize whitespace
        let normalized = result
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        self.truncate(&normalized)
    }

    /// Compress general text: normalize whitespace, truncate
    pub fn compress_text(&self, text: &str) -> String {
        let normalized = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        self.truncate(&normalized)
    }

    /// Shorten URLs to domain+path (remove query params)
    pub fn shorten_urls(&self, text: &str) -> String {
        // Simple URL shortening: keep scheme + host + path, drop most query params
        text.split_whitespace()
            .map(|word| {
                if word.starts_with("http://") || word.starts_with("https://") {
                    if let Some(stripped) = word.split('?').next() {
                        return stripped.to_string();
                    }
                }
                word.to_string()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn truncate(&self, text: &str) -> String {
        if text.chars().count() <= self.max_chars {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(self.max_chars).collect();
            format!(
                "{}... [truncated, {} total chars]",
                truncated,
                text.chars().count()
            )
        }
    }
}

impl Default for TokenCompressor {
    fn default() -> Self {
        Self { max_chars: 8000 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_html_tags() {
        let c = TokenCompressor::new(1000);
        let result = c.compress_html("<p>Hello <b>World</b></p>");
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
        assert!(!result.contains("<p>"));
        assert!(!result.contains("<b>"));
    }

    #[test]
    fn normalizes_whitespace() {
        let c = TokenCompressor::new(1000);
        let result = c.compress_text("  line one  \n\n  line two  \n  ");
        assert_eq!(result, "line one\nline two");
    }

    #[test]
    fn shortens_urls() {
        let c = TokenCompressor::new(1000);
        let result = c.shorten_urls("see https://example.com/page?utm=track&ref=ad");
        assert!(!result.contains("utm"));
        assert!(result.contains("example.com/page"));
    }

    #[test]
    fn truncates_long_text() {
        let c = TokenCompressor::new(10);
        let result = c.compress_text("this is a very long text that should be truncated");
        assert!(result.len() <= 50); // 10 chars + truncation suffix
        assert!(result.contains("truncated"));
    }

    #[test]
    fn strips_script_and_style_tags() {
        let c = TokenCompressor::new(1000);
        let html = "<html><body><script>var x = 1; alert('xss');</script><p>Safe content</p><style>body { color: red; }</style><p>More</p></body></html>";
        let result = c.compress_html(html);
        assert!(result.contains("Safe content"));
        assert!(result.contains("More"));
        assert!(!result.contains("alert"));
        assert!(!result.contains("color: red"));
    }

    #[test]
    fn preserves_non_html_text() {
        let c = TokenCompressor::new(1000);
        let result = c.compress_text("plain text\nwith lines");
        assert_eq!(result, "plain text\nwith lines");
    }

    #[test]
    fn shorten_urls_preserves_non_urls() {
        let c = TokenCompressor::new(1000);
        let result = c.shorten_urls("hello world");
        assert_eq!(result, "hello world");
    }
}
