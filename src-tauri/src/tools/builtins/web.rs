//! Web 工具模块
//!
//! web_search 和 web_fetch 工具。
//! 使用 DuckDuckGo Instant Answer API 进行搜索，reqwest 直接获取网页内容。

use crate::tools::executor::{Tool, ToolError};
use regex_lite::Regex;
use serde::Deserialize;
use std::pin::Pin;
use std::time::Duration;

#[derive(Deserialize)]
struct SearchParams {
    query: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    5
}

pub struct WebSearchTool;

impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "搜索网页信息，返回搜索结果摘要。可搜索天气、新闻、百科知识等。"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "搜索关键词" },
                "limit": { "type": "number", "description": "结果数量限制（默认5）" }
            },
            "required": ["query"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<SearchParams>(input).is_ok()
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let input = input.to_string();
        Box::pin(async move {
            let params: SearchParams = serde_json::from_str(&input)
                .map_err(|e| ToolError::ValidationFailed(e.to_string()))?;

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent("FairyField/1.0")
                .build()
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            // 优先使用 DuckDuckGo Instant Answer API；如果它没有通用网页结果，
            // 再回退到 DuckDuckGo Lite HTML 搜索页。
            let mut url = reqwest::Url::parse("https://api.duckduckgo.com/")
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
            url.query_pairs_mut()
                .append_pair("q", &params.query)
                .append_pair("format", "json")
                .append_pair("no_html", "1")
                .append_pair("skip_disambig", "1");

            let response = client.get(url).send().await;
            let json: serde_json::Value = match response {
                Ok(resp) => match resp.json().await {
                    Ok(json) => json,
                    Err(e) => {
                        if let Some(lite) =
                            try_duckduckgo_lite_search(&client, &params.query, params.limit).await
                        {
                            return Ok(lite);
                        }
                        return Err(ToolError::ExecutionFailed(format!(
                            "解析搜索结果失败: {}",
                            e
                        )));
                    }
                },
                Err(e) => {
                    if let Some(lite) =
                        try_duckduckgo_lite_search(&client, &params.query, params.limit).await
                    {
                        return Ok(lite);
                    }
                    return Ok(search_fallback_links(
                        &params.query,
                        Some(&format!("DuckDuckGo 搜索不可用（网络限制: {}）", e)),
                    ));
                }
            };

            let mut results = Vec::new();

            // 主要摘要（如果有 Instant Answer）
            if let Some(abstract_text) = json.get("AbstractText").and_then(|v| v.as_str()) {
                if !abstract_text.is_empty() {
                    let source = json
                        .get("AbstractSource")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    results.push(format!("📋 {}（来源: {}）", abstract_text, source));
                }
            }

            // 相关主题
            if let Some(topics) = json.get("RelatedTopics").and_then(|v| v.as_array()) {
                let mut count = 0;
                for topic in topics.iter() {
                    if count >= params.limit as usize {
                        break;
                    }
                    // 普通主题
                    if let Some(text) = topic.get("Text").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            results.push(format!("• {}", text));
                            count += 1;
                        }
                    }
                    // 子主题（嵌套在 Topics 数组中）
                    if let Some(sub_topics) = topic.get("Topics").and_then(|v| v.as_array()) {
                        for sub in sub_topics.iter() {
                            if count >= params.limit as usize {
                                break;
                            }
                            if let Some(text) = sub.get("Text").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    results.push(format!("• {}", text));
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }

            // 如果 DuckDuckGo 没有返回结果，尝试 wttr.in（天气专用）
            if results.is_empty() {
                if let Some(lite) =
                    try_duckduckgo_lite_search(&client, &params.query, params.limit).await
                {
                    results.push(lite);
                }
            }

            if results.is_empty() {
                let weather_result = try_weather_search(&client, &params.query).await;
                if let Some(weather) = weather_result {
                    results.push(weather);
                }
            }

            if results.is_empty() {
                Ok(search_fallback_links(&params.query, None))
            } else {
                Ok(results.join("\n\n"))
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

async fn try_duckduckgo_lite_search(
    client: &reqwest::Client,
    query: &str,
    limit: u32,
) -> Option<String> {
    let mut url = reqwest::Url::parse("https://lite.duckduckgo.com/lite/").ok()?;
    url.query_pairs_mut().append_pair("q", query);

    let html = client.get(url).send().await.ok()?.text().await.ok()?;
    let results = parse_duckduckgo_lite_results(&html, limit as usize);
    if results.is_empty() {
        return None;
    }

    let formatted = results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            if result.snippet.is_empty() {
                format!("{}. {}\n{}", i + 1, result.title, result.url)
            } else {
                format!(
                    "{}. {}\n{}\n{}",
                    i + 1,
                    result.title,
                    result.url,
                    result.snippet
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Some(format!("🔎 搜索结果（DuckDuckGo Lite）\n\n{}", formatted))
}

fn parse_duckduckgo_lite_results(html: &str, limit: usize) -> Vec<SearchResult> {
    let anchor_re =
        Regex::new(r#"(?is)<a\b[^>]*class=["'][^"']*result-link[^"']*["'][^>]*>(.*?)</a>"#)
            .expect("valid result-link regex");
    let href_re = Regex::new(r#"(?is)href=["']([^"']+)["']"#).expect("valid href regex");
    let snippet_re =
        Regex::new(r#"(?is)<td\b[^>]*class=["'][^"']*result-snippet[^"']*["'][^>]*>(.*?)</td>"#)
            .expect("valid snippet regex");

    let snippets = snippet_re
        .captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| html_to_text(m.as_str())))
        .collect::<Vec<_>>();

    let mut results = Vec::new();
    for (index, cap) in anchor_re.captures_iter(html).enumerate() {
        if results.len() >= limit {
            break;
        }
        let anchor_html = match cap.get(0) {
            Some(m) => m.as_str(),
            None => continue,
        };
        let href = match href_re
            .captures(anchor_html)
            .and_then(|href_cap| href_cap.get(1))
        {
            Some(m) => normalize_duckduckgo_url(m.as_str()),
            None => continue,
        };
        let title = cap
            .get(1)
            .map(|m| html_to_text(m.as_str()))
            .unwrap_or_default();
        if title.is_empty() || href.is_empty() {
            continue;
        }

        results.push(SearchResult {
            title,
            url: href,
            snippet: snippets.get(index).cloned().unwrap_or_default(),
        });
    }

    results
}

fn normalize_duckduckgo_url(raw_href: &str) -> String {
    let decoded = decode_html_entities(raw_href);
    let candidate = if decoded.starts_with("//") {
        format!("https:{}", decoded)
    } else if decoded.starts_with('/') {
        format!("https://duckduckgo.com{}", decoded)
    } else {
        decoded
    };

    if let Ok(url) = reqwest::Url::parse(&candidate) {
        if let Some((_, uddg)) = url.query_pairs().find(|(key, _)| key == "uddg") {
            return uddg.into_owned();
        }
        return url.to_string();
    }

    candidate
}

fn html_to_text(html: &str) -> String {
    decode_html_entities(&strip_html_tags(html))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

fn encoded_query_component(query: &str) -> String {
    let mut url = reqwest::Url::parse("https://example.com/").expect("valid base url");
    url.query_pairs_mut().append_pair("q", query);
    url.query()
        .and_then(|q| q.strip_prefix("q="))
        .unwrap_or("")
        .to_string()
}

fn search_fallback_links(query: &str, reason: Option<&str>) -> String {
    let encoded = encoded_query_component(query);
    let prefix = reason
        .map(|r| format!("⚠️ {}。\n\n", r))
        .unwrap_or_else(|| format!("未找到「{}」的即时搜索结果。\n\n", query));
    format!(
        "{}你可以通过以下链接继续搜索：\n• Google: https://www.google.com/search?q={}\n• Bing: https://www.bing.com/search?q={}\n• 百度: https://www.baidu.com/s?wd={}",
        prefix, encoded, encoded, encoded
    )
}

/// 尝试通过 wttr.in 获取天气信息
async fn try_weather_search(client: &reqwest::Client, query: &str) -> Option<String> {
    // 检测是否包含天气相关关键词
    let weather_keywords = ["天气", "weather", "气温", "温度", "下雨", "晴"];
    let is_weather = weather_keywords
        .iter()
        .any(|k| query.to_lowercase().contains(k));

    if !is_weather {
        return None;
    }

    // 从查询中提取城市名
    let city = extract_city(query)?;

    let url = format!("https://wttr.in/{}?format=j1", city);

    let response = client.get(&url).send().await.ok()?;
    let json: serde_json::Value = response.json().await.ok()?;

    let current = json.get("current_condition")?.as_array()?.first()?;

    let temp = current.get("temp_C")?.as_str()?;
    let desc = current
        .get("lang_zh")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            current
                .get("weatherDesc")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("未知");
    let humidity = current.get("humidity")?.as_str()?;
    let wind = current.get("windspeedKmph")?.as_str()?;

    Some(format!(
        "🌤️ {} 当前天气: {}，温度 {}°C，湿度 {}%，风速 {} km/h",
        city, desc, temp, humidity, wind
    ))
}

/// 从查询中提取城市名
fn extract_city(query: &str) -> Option<String> {
    // 移除天气相关关键词，提取城市名
    let weather_words = [
        "天气",
        "weather",
        "气温",
        "温度",
        "怎么样",
        "如何",
        "如何了",
        "下雨",
        "晴",
        "阴",
        "多云",
        "的",
        "今天",
        "明天",
        "后天",
        "现在",
        "请问",
        "查一下",
        "看看",
    ];

    let mut city = query.to_string();
    for word in &weather_words {
        city = city.replace(word, "");
    }

    let city = city.trim().to_string();

    if city.is_empty() {
        // 默认返回北京
        Some("Beijing".to_string())
    } else {
        // 将常见中文城市名映射为英文
        let city_map = [
            ("北京", "Beijing"),
            ("上海", "Shanghai"),
            ("广州", "Guangzhou"),
            ("深圳", "Shenzhen"),
            ("成都", "Chengdu"),
            ("杭州", "Hangzhou"),
            ("南京", "Nanjing"),
            ("武汉", "Wuhan"),
            ("西安", "Xi'an"),
            ("重庆", "Chongqing"),
            ("天津", "Tianjin"),
            ("苏州", "Suzhou"),
            ("长沙", "Changsha"),
            ("青岛", "Qingdao"),
            ("大连", "Dalian"),
        ];

        for (cn, en) in &city_map {
            if city.contains(cn) {
                return Some(en.to_string());
            }
        }

        Some(city)
    }
}

#[derive(Deserialize)]
struct FetchParams {
    url: String,
}

pub struct WebFetchTool;

impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }
    fn description(&self) -> &str {
        "获取指定 URL 的网页内容，返回纯文本。"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "目标 URL" }
            },
            "required": ["url"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<FetchParams>(input).is_ok()
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let input = input.to_string();
        Box::pin(async move {
            let params: FetchParams = serde_json::from_str(&input)
                .map_err(|e| ToolError::ValidationFailed(e.to_string()))?;

            // 基本 URL 安全检查
            if !params.url.starts_with("http://") && !params.url.starts_with("https://") {
                return Err(ToolError::ValidationFailed(
                    "URL 必须以 http:// 或 https:// 开头".to_string(),
                ));
            }

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(20))
                .user_agent("FairyField/1.0")
                .build()
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            let response = client
                .get(&params.url)
                .send()
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("请求失败: {}", e)))?;

            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let text = response
                .text()
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("读取内容失败: {}", e)))?;

            // 简单的 HTML 标签剥离（如果内容是 HTML）
            let cleaned = if content_type.contains("html") {
                strip_html_tags(&text)
            } else {
                text
            };

            // 截断到合理大小
            let truncated = if cleaned.len() > 5000 {
                format!(
                    "{}...\n\n[内容已截断，原始长度: {} 字符]",
                    &cleaned[..5000],
                    cleaned.len()
                )
            } else {
                cleaned
            };

            Ok(truncated)
        })
    }
}

/// 简单的 HTML 标签剥离（跳过 script/style 内容）
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut skip_content = false; // 跳过 script/style 标签内容
    let mut tag_name = String::new();
    let mut tag_name_done = false;

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '<' => {
                in_tag = true;
                tag_name.clear();
                tag_name_done = false;
            }
            '>' => {
                in_tag = false;
                // 检查是否进入 script/style 块
                let name = tag_name.to_lowercase();
                if name.starts_with("script") || name.starts_with("style") {
                    skip_content = true;
                }
                // 检查是否结束 script/style 块
                if name.starts_with("/script") || name.starts_with("/style") {
                    skip_content = false;
                }
            }
            _ => {
                if in_tag {
                    if !tag_name_done {
                        if ch.is_alphabetic() || ch == '/' {
                            tag_name.push(ch);
                        } else {
                            tag_name_done = true;
                        }
                    }
                } else if !skip_content {
                    result.push(ch);
                }
            }
        }
        i += 1;
    }

    // 清理多余空白
    let cleaned: String = result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_search_input() {
        let tool = WebSearchTool;
        assert!(tool.validate_input(r#"{"query": "rust"}"#));
        assert!(!tool.validate_input("not json"));
    }

    #[tokio::test]
    async fn execute_search_returns_result() {
        let tool = WebSearchTool;
        let result = tool.execute(r#"{"query": "rust", "limit": 3}"#).await;
        // 网络测试：结果可能是搜索结果或错误（无网络时）
        match result {
            Ok(text) => {
                // 应该包含有意义的内容，不再是 mock
                assert!(!text.contains("web_search mock"));
            }
            Err(_) => {
                // 网络不可用时允许失败
            }
        }
    }

    #[tokio::test]
    async fn execute_search_known_query_returns_usable_output() {
        let tool = WebSearchTool;
        let result = tool
            .execute(r#"{"query": "Rust programming language", "limit": 3}"#)
            .await
            .unwrap();
        println!("web_search smoke output:\n{}", result);
        assert!(!result.contains("web_search mock"));
        assert!(!result.contains("未找到「Rust programming language」"));
        assert!(
            result.contains("Rust") || result.contains("搜索结果"),
            "unexpected search output: {result}"
        );
    }

    #[test]
    fn validate_fetch_input() {
        let tool = WebFetchTool;
        assert!(tool.validate_input(r#"{"url": "https://example.com"}"#));
        // validate_input 只检查 JSON 格式，URL 协议检查在 execute() 中
        assert!(tool.validate_input(r#"{"url": "ftp://bad"}"#));
        assert!(!tool.validate_input("not json"));
    }

    #[tokio::test]
    async fn execute_fetch_validates_url() {
        let tool = WebFetchTool;
        let result = tool.execute(r#"{"url": "ftp://bad.example.com"}"#).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_city() {
        assert_eq!(extract_city("北京天气"), Some("Beijing".to_string()));
        assert_eq!(extract_city("上海天气怎么样"), Some("Shanghai".to_string()));
        assert_eq!(extract_city("今天天气如何"), Some("Beijing".to_string()));
    }

    #[test]
    fn test_strip_html() {
        let html = "<html><body><h1>Hello</h1><p>World</p></body></html>";
        let text = strip_html_tags(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("<"));
    }

    #[test]
    fn test_strip_html_skips_script() {
        let html =
            "<html><body><script>var x = 1; alert('hi');</script><p>Visible</p></body></html>";
        let text = strip_html_tags(html);
        assert!(text.contains("Visible"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("var x"));
    }

    #[test]
    fn test_strip_html_skips_style() {
        let html = "<html><body><style>body { color: red; }</style><p>Content</p></body></html>";
        let text = strip_html_tags(html);
        assert!(text.contains("Content"));
        assert!(!text.contains("color"));
    }

    #[test]
    fn parse_duckduckgo_lite_results_extracts_links_and_snippets() {
        let html = r#"
            <a rel="nofollow" class='result-link' href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Ffairy%3Fa%3D1%26b%3D2">Fairy &amp; Field</a>
            <td class='result-snippet'>A <b>desktop</b> companion.</td>
        "#;

        let results = parse_duckduckgo_lite_results(html, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Fairy & Field");
        assert_eq!(results[0].url, "https://example.com/fairy?a=1&b=2");
        assert_eq!(results[0].snippet, "A desktop companion.");
    }

    #[test]
    fn fallback_links_percent_encode_chinese_queries() {
        let links = search_fallback_links("澳门 天气", None);
        assert!(links.contains("%E6%BE%B3%E9%97%A8+%E5%A4%A9%E6%B0%94"));
        assert!(!links.contains("q=澳门"));
    }
}
