//! Web 工具模块
//!
//! web_search 和 web_fetch 工具。
//! 使用 DuckDuckGo Instant Answer API 进行搜索，reqwest 直接获取网页内容。

use crate::text::truncate_chars_with_suffix;
use crate::tools::executor::{Tool, ToolError};
use futures_util::StreamExt;
use regex_lite::Regex;
use reqwest::header::LOCATION;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::pin::Pin;
use std::time::Duration;

const SEARCH_TIMEOUT_SECS: u64 = 8;
const FETCH_TIMEOUT_SECS: u64 = 12;
const WEATHER_TIMEOUT_SECS: u64 = 6;
const DNS_TIMEOUT_SECS: u64 = 3;
const MAX_SEARCH_RESULTS: u32 = 5;
const MAX_QUERY_CHARS: usize = 200;
const MAX_FETCH_BYTES: usize = 256 * 1024;
const MAX_FETCH_CHARS: usize = 4_000;
const MAX_REDIRECTS: usize = 5;

#[derive(Deserialize)]
struct SearchParams {
    query: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    5
}

fn validate_search_params(params: &SearchParams) -> Result<(), String> {
    let query = params.query.trim();
    if query.is_empty() {
        return Err("搜索关键词不能为空".to_string());
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(format!("搜索关键词不能超过 {MAX_QUERY_CHARS} 个字符"));
    }
    if params.limit == 0 {
        return Err("结果数量必须大于 0".to_string());
    }
    Ok(())
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
        serde_json::from_str::<SearchParams>(input)
            .map(|params| validate_search_params(&params).is_ok())
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let input = input.to_string();
        Box::pin(async move {
            let params: SearchParams = serde_json::from_str(&input)
                .map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            validate_search_params(&params).map_err(ToolError::ValidationFailed)?;
            let limit = params.limit.clamp(1, MAX_SEARCH_RESULTS);

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(SEARCH_TIMEOUT_SECS))
                .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                .user_agent("FairyField/1.0")
                .build()
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            if is_weather_query(&params.query) {
                if let Some(weather) = try_weather_search(&client, &params.query).await {
                    return Ok(weather);
                }
                return Ok(weather_unavailable_message(&params.query));
            }

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
                        if let Some(instant) =
                            try_duckduckgo_instant_answer_with_curl(&params.query, limit).await
                        {
                            return Ok(instant);
                        }
                        if let Some(lite) =
                            try_duckduckgo_lite_search(&client, &params.query, limit).await
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
                    if let Some(instant) =
                        try_duckduckgo_instant_answer_with_curl(&params.query, limit).await
                    {
                        return Ok(instant);
                    }
                    if let Some(lite) =
                        try_duckduckgo_lite_search(&client, &params.query, limit).await
                    {
                        return Ok(lite);
                    }
                    return Ok(search_fallback_links(
                        &params.query,
                        Some(&format!("DuckDuckGo 搜索不可用（网络限制: {}）", e)),
                    ));
                }
            };

            let mut results = format_duckduckgo_instant_results(&json, limit);

            // 如果 DuckDuckGo 没有返回结果，尝试 wttr.in（天气专用）
            if results.is_empty() {
                if let Some(lite) = try_duckduckgo_lite_search(&client, &params.query, limit).await
                {
                    results.push(lite);
                }
            }

            if results.is_empty() {
                if let Some(weather) = try_weather_search(&client, &params.query).await {
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

async fn try_duckduckgo_instant_answer_with_curl(query: &str, limit: u32) -> Option<String> {
    let mut url = reqwest::Url::parse("https://api.duckduckgo.com/").ok()?;
    url.query_pairs_mut()
        .append_pair("q", query)
        .append_pair("format", "json")
        .append_pair("no_html", "1")
        .append_pair("skip_disambig", "1");

    let json = fetch_json_with_curl(url.as_str(), SEARCH_TIMEOUT_SECS).await?;
    let results = format_duckduckgo_instant_results(&json, limit);
    if results.is_empty() {
        None
    } else {
        Some(results.join("\n\n"))
    }
}

fn format_duckduckgo_instant_results(json: &serde_json::Value, limit: u32) -> Vec<String> {
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
            if count >= limit as usize {
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
                    if count >= limit as usize {
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

    results
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

fn weather_unavailable_message(query: &str) -> String {
    let encoded = encoded_query_component(query);
    format!(
        "🌤️ 暂时无法获取「{}」的实时天气数据。\n\n可以稍后重试，或打开以下天气搜索链接：\n• Google: https://www.google.com/search?q={}\n• Bing: https://www.bing.com/search?q={}\n• 百度: https://www.baidu.com/s?wd={}",
        query, encoded, encoded, encoded
    )
}

/// 尝试通过 wttr.in 获取天气信息
async fn try_weather_search(client: &reqwest::Client, query: &str) -> Option<String> {
    if !is_weather_query(query) {
        return None;
    }

    // 从查询中提取城市名
    let city = extract_city(query)?;

    if let Some(weather) = try_open_meteo_weather(client, &city).await {
        return Some(weather);
    }

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

async fn try_open_meteo_weather(client: &reqwest::Client, city: &str) -> Option<String> {
    let (display, latitude, longitude) = city_coordinates(city)?;
    let mut url = reqwest::Url::parse("https://api.open-meteo.com/v1/forecast").ok()?;
    url.query_pairs_mut()
        .append_pair("latitude", &latitude.to_string())
        .append_pair("longitude", &longitude.to_string())
        .append_pair(
            "current",
            "temperature_2m,relative_humidity_2m,precipitation,weather_code,wind_speed_10m",
        )
        .append_pair("timezone", "auto");

    let reqwest_json = async {
        client
            .get(url.clone())
            .send()
            .await
            .ok()?
            .json::<serde_json::Value>()
            .await
            .ok()
    };
    if let Ok(Some(json)) =
        tokio::time::timeout(Duration::from_secs(WEATHER_TIMEOUT_SECS), reqwest_json).await
    {
        if let Some(weather) = format_open_meteo_weather(display, &json) {
            return Some(weather);
        }
    }

    let json = fetch_json_with_curl(url.as_str(), WEATHER_TIMEOUT_SECS).await?;
    format_open_meteo_weather(display, &json)
}

async fn fetch_json_with_curl(url: &str, timeout_secs: u64) -> Option<serde_json::Value> {
    let mut command = tokio::process::Command::new("curl");
    let timeout_arg = timeout_secs.to_string();
    command.args([
        "--location",
        "--silent",
        "--show-error",
        "--fail",
        "--max-time",
        &timeout_arg,
        url,
    ]);

    let output = tokio::time::timeout(Duration::from_secs(timeout_secs + 1), command.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

fn format_open_meteo_weather(display: &str, json: &serde_json::Value) -> Option<String> {
    let current = json.get("current")?;
    let temp = current.get("temperature_2m")?.as_f64()?;
    let humidity = current.get("relative_humidity_2m")?.as_f64()?;
    let precipitation = current
        .get("precipitation")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let wind = current.get("wind_speed_10m")?.as_f64()?;
    let code = current
        .get("weather_code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    let desc = weather_code_description(code);

    Some(format!(
        "🌤️ {display} 当前天气: {desc}，温度 {:.1}°C，湿度 {:.0}%，降水 {:.1} mm，风速 {:.1} km/h",
        temp, humidity, precipitation, wind
    ))
}

fn city_coordinates(city: &str) -> Option<(&'static str, f64, f64)> {
    match city.to_ascii_lowercase().as_str() {
        "macau" | "macao" => Some(("澳门", 22.1987, 113.5439)),
        "hong kong" | "hongkong" => Some(("香港", 22.3193, 114.1694)),
        "beijing" => Some(("北京", 39.9042, 116.4074)),
        "shanghai" => Some(("上海", 31.2304, 121.4737)),
        "guangzhou" => Some(("广州", 23.1291, 113.2644)),
        "shenzhen" => Some(("深圳", 22.5431, 114.0579)),
        "chengdu" => Some(("成都", 30.5728, 104.0668)),
        "hangzhou" => Some(("杭州", 30.2741, 120.1551)),
        "nanjing" => Some(("南京", 32.0603, 118.7969)),
        "wuhan" => Some(("武汉", 30.5928, 114.3055)),
        "xi'an" | "xian" => Some(("西安", 34.3416, 108.9398)),
        "chongqing" => Some(("重庆", 29.5630, 106.5516)),
        "tianjin" => Some(("天津", 39.3434, 117.3616)),
        "suzhou" => Some(("苏州", 31.2989, 120.5853)),
        "changsha" => Some(("长沙", 28.2282, 112.9388)),
        "qingdao" => Some(("青岛", 36.0671, 120.3826)),
        "dalian" => Some(("大连", 38.9140, 121.6147)),
        "tokyo" => Some(("东京", 35.6762, 139.6503)),
        "new york" | "newyork" => Some(("New York", 40.7128, -74.0060)),
        "london" => Some(("London", 51.5072, -0.1276)),
        _ => None,
    }
}

fn weather_code_description(code: i64) -> &'static str {
    match code {
        0 => "晴朗",
        1..=3 => "多云",
        45 | 48 => "有雾",
        51 | 53 | 55 | 56 | 57 => "毛毛雨",
        61 | 63 | 65 | 66 | 67 => "降雨",
        71 | 73 | 75 | 77 => "降雪",
        80..=82 => "阵雨",
        85 | 86 => "阵雪",
        95 | 96 | 99 => "雷暴",
        _ => "未知",
    }
}

fn is_weather_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    ["天气", "weather", "气温", "温度", "下雨", "晴", "forecast"]
        .iter()
        .any(|keyword| lower.contains(keyword))
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
            ("澳门", "Macau"),
            ("澳門", "Macau"),
            ("香港", "Hong Kong"),
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
        serde_json::from_str::<FetchParams>(input)
            .map(|params| validate_public_http_url(&params.url).is_ok())
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let input = input.to_string();
        Box::pin(async move {
            let params: FetchParams = serde_json::from_str(&input)
                .map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            validate_public_http_url(&params.url).map_err(ToolError::ValidationFailed)?;

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
                .redirect(reqwest::redirect::Policy::none())
                .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
                .user_agent("FairyField/1.0")
                .build()
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            validate_public_http_url_async(&params.url)
                .await
                .map_err(ToolError::ValidationFailed)?;

            let response = fetch_with_safe_redirects(&client, &params.url).await?;

            let status = response.status();
            if !status.is_success() {
                return Err(ToolError::ExecutionFailed(format!(
                    "请求失败，HTTP 状态码: {}",
                    status
                )));
            }

            let content_type = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let (text, body_truncated) = read_response_text_limited(response, MAX_FETCH_BYTES)
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("读取内容失败: {e}")))?;

            // 简单的 HTML 标签剥离（如果内容是 HTML）
            let cleaned = if content_type.contains("html") {
                html_to_text(&text)
            } else {
                text
            };

            // 截断到合理大小
            let mut truncated = truncate_chars_with_suffix(&cleaned, MAX_FETCH_CHARS, "...");
            if body_truncated || truncated.len() < cleaned.len() {
                truncated.push_str(&format!(
                    "\n\n[内容已截断，返回前 {} 字符；响应读取上限 {} 字节]",
                    MAX_FETCH_CHARS, MAX_FETCH_BYTES
                ));
            }

            Ok(truncated)
        })
    }
}

fn validate_public_http_url(raw_url: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(raw_url).map_err(|e| format!("URL 格式无效: {e}"))?;
    validate_public_http_url_parts(&url)
}

fn validate_public_http_url_parts(url: &reqwest::Url) -> Result<(), String> {
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("URL 必须以 http:// 或 https:// 开头".to_string()),
    }

    let host = url
        .host_str()
        .ok_or_else(|| "URL host is required".to_string())?;
    let host_lower = host.to_ascii_lowercase();
    if matches!(host_lower.as_str(), "localhost" | "0.0.0.0") || host_lower.ends_with(".local") {
        return Err("Local hosts are not allowed".to_string());
    }
    if let Ok(ip) = host_lower.parse::<IpAddr>() {
        if is_private_or_local_ip(ip) {
            return Err("Private or local IP addresses are not allowed".to_string());
        }
    }
    Ok(())
}

async fn validate_public_http_url_async(raw_url: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(raw_url).map_err(|e| format!("URL 格式无效: {e}"))?;
    validate_public_http_url_parts(&url)?;

    let host = url
        .host_str()
        .ok_or_else(|| "URL host is required".to_string())?;
    if host.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    let port = url.port_or_known_default().unwrap_or(80);
    let lookup = tokio::net::lookup_host((host, port));
    let addrs = tokio::time::timeout(Duration::from_secs(DNS_TIMEOUT_SECS), lookup)
        .await
        .map_err(|_| "DNS lookup timed out".to_string())?
        .map_err(|e| format!("DNS lookup failed: {e}"))?;

    for addr in addrs.take(8) {
        if is_private_or_local_ip(addr.ip()) {
            return Err("Host resolves to a private or local address".to_string());
        }
    }
    Ok(())
}

async fn fetch_with_safe_redirects(
    client: &reqwest::Client,
    start_url: &str,
) -> Result<reqwest::Response, ToolError> {
    let mut current = reqwest::Url::parse(start_url)
        .map_err(|e| ToolError::ValidationFailed(format!("URL 格式无效: {e}")))?;

    for redirect_count in 0..=MAX_REDIRECTS {
        validate_public_http_url_async(current.as_str())
            .await
            .map_err(ToolError::ValidationFailed)?;

        let response = client
            .get(current.clone())
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("请求失败: {}", e)))?;

        if !response.status().is_redirection() {
            return Ok(response);
        }

        if redirect_count >= MAX_REDIRECTS {
            return Err(ToolError::ExecutionFailed(format!(
                "重定向次数超过 {} 次",
                MAX_REDIRECTS
            )));
        }

        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ToolError::ExecutionFailed("重定向缺少 Location 头".to_string()))?;
        let next = current
            .join(location)
            .map_err(|e| ToolError::ValidationFailed(format!("重定向地址无效: {e}")))?;
        validate_public_http_url_parts(&next)
            .map_err(|e| ToolError::ValidationFailed(format!("重定向到不安全地址: {e}")))?;
        current = next;
    }

    Err(ToolError::ExecutionFailed(format!(
        "重定向次数超过 {} 次",
        MAX_REDIRECTS
    )))
}

fn is_private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private() || v4.is_loopback() || v4.is_link_local() || v4.is_unspecified()
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local(),
    }
}

async fn read_response_text_limited(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<(String, bool), reqwest::Error> {
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    let mut truncated = false;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let remaining = max_bytes.saturating_sub(bytes.len());
        if chunk.len() > remaining {
            bytes.extend_from_slice(&chunk[..remaining]);
            truncated = true;
            break;
        }
        bytes.extend_from_slice(&chunk);
        if bytes.len() >= max_bytes {
            truncated = true;
            break;
        }
    }

    let text = String::from_utf8_lossy(&bytes).to_string();
    Ok((text, truncated))
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
        assert!(!tool.validate_input(r#"{"query": ""}"#));
        assert!(!tool.validate_input(r#"{"query": "rust", "limit": 0}"#));
        assert!(!tool.validate_input("not json"));
    }

    #[test]
    fn detects_weather_queries_before_general_search() {
        assert!(is_weather_query("澳门天气"));
        assert!(is_weather_query("weather in Tokyo"));
        assert!(!is_weather_query("Rust programming language"));
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
        assert!(!tool.validate_input(r#"{"url": "ftp://bad"}"#));
        assert!(!tool.validate_input(r#"{"url": "http://localhost:1420"}"#));
        assert!(!tool.validate_input(r#"{"url": "http://127.0.0.1"}"#));
        assert!(!tool.validate_input(r#"{"url": "http://192.168.1.1"}"#));
        assert!(!tool.validate_input("not json"));
    }

    #[tokio::test]
    async fn execute_fetch_validates_url() {
        let tool = WebFetchTool;
        let result = tool.execute(r#"{"url": "ftp://bad.example.com"}"#).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_fetch_rejects_localhost_before_network() {
        let tool = WebFetchTool;
        let result = tool.execute(r#"{"url": "http://localhost:1420"}"#).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Local hosts"));
    }

    #[test]
    fn test_extract_city() {
        assert_eq!(extract_city("北京天气"), Some("Beijing".to_string()));
        assert_eq!(extract_city("上海天气怎么样"), Some("Shanghai".to_string()));
        assert_eq!(extract_city("澳门天气"), Some("Macau".to_string()));
        assert_eq!(extract_city("今天天气如何"), Some("Beijing".to_string()));
    }

    #[tokio::test]
    async fn execute_weather_query_prefers_weather_result() {
        let tool = WebSearchTool;
        let result = tool
            .execute(r#"{"query": "澳门天气", "limit": 3}"#)
            .await
            .unwrap();
        println!("weather smoke output:\n{}", result);
        assert!(
            result.contains("当前天气") || result.contains("Macau"),
            "unexpected weather output: {result}"
        );
        assert!(
            !result.contains("DuckDuckGo Lite"),
            "weather query should not prefer generic web results: {result}"
        );
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
