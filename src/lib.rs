// This project is based on Link Cleaner by Corbin Davenport, which is licensed
// under the GNU General Public License v3.0. The original source code can be
// found at https://github.com/corbindavenport/link-cleaner.

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use url::Url;

pub mod settings;

static URL_EXTRACTOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://\S+").expect("valid regex pattern"));

static AMAZON_PRODUCT_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:/dp/|/product/|/d/)(\w+|\d+)").expect("valid regex pattern"));

static BESTBUY_SKU_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/(\d+)\.p").expect("valid regex pattern"));

/// Settings controlling optional link cleaning behavior.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    /// Convert YouTube video links to the `youtu.be` short format.
    pub youtube_shorten: bool,
    /// Replace `twitter.com` / `x.com` with `fxtwitter.com`.
    pub fix_twitter: bool,
    /// Replace `bsky.app` with `fxbsky.app`.
    pub fix_bluesky: bool,
}

/// Cleans a URL by removing tracking parameters and applying optional transforms.
///
/// This unwraps known redirect services (l.facebook.com, href.li, google.com/url,
/// cts.businesswire.com), strips tracking query parameters, and preserves site-specific
/// essential parameters (YouTube video IDs, Amazon product IDs, etc.).
pub fn clean_link(input: &str, settings: &Settings) -> Result<String> {
    let mut url = extract_url(input)?;

    for _ in 0..5 {
        let unwrapped = match url.host_str() {
            Some("l.facebook.com") => query_param(&url, "u"),
            Some("href.li") => url.query().map(|q| q.to_string()),
            Some("www.google.com") if url.path() == "/url" => query_param(&url, "url"),
            Some("cts.businesswire.com") => query_param(&url, "url"),
            _ => None,
        };
        match unwrapped {
            Some(u) => match Url::parse(&u) {
                Ok(parsed) => url = parsed,
                Err(_) => break,
            },
            None => break,
        }
    }

    let params: HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let host = url.host_str().unwrap_or("").to_lowercase();

    let mut cleaned = url.clone();
    cleaned.set_query(None);
    cleaned.set_fragment(None);

    // --- always-preserved parameters ---

    if let Some(q) = params.get("q") {
        cleaned.query_pairs_mut().append_pair("q", q);
    }
    if host == "play.google.com"
        && let Some(id) = params.get("id")
    {
        cleaned.query_pairs_mut().append_pair("id", id);
    }
    if host == "www.macys.com"
        && let Some(id) = params.get("ID")
    {
        cleaned.query_pairs_mut().append_pair("ID", id);
    }

    // --- YouTube ---

    let is_youtube = host == "youtube.com" || host.ends_with(".youtube.com");
    let is_youtu_be = host == "youtu.be";

    if is_youtube && params.contains_key("v") {
        if settings.youtube_shorten {
            if let Some(v) = params.get("v") {
                cleaned =
                    Url::parse(&format!("https://youtu.be/{v}")).context("failed to construct youtu.be short link")?;
            }
        } else if let Some(v) = params.get("v") {
            cleaned.query_pairs_mut().append_pair("v", v);
        }
        if let Some(t) = params.get("t") {
            cleaned.query_pairs_mut().append_pair("t", t);
        }
    } else if is_youtube && url.path().contains("playlist") && params.contains_key("list") {
        if let Some(list) = params.get("list") {
            cleaned.query_pairs_mut().append_pair("list", list);
        }
    } else if is_youtu_be && let Some(t) = params.get("t") {
        cleaned.query_pairs_mut().append_pair("t", t);
    }

    // --- Facebook story ---

    if host == "www.facebook.com" && url.path().contains("story.php") {
        if let Some(story) = params.get("story_fbid") {
            cleaned.query_pairs_mut().append_pair("story_fbid", story);
        }
        if let Some(id) = params.get("id") {
            cleaned.query_pairs_mut().append_pair("id", id);
        }
    }

    // --- Amazon product pages ---

    if is_amazon_host(&host)
        && (url.path().contains("/dp/") || url.path().contains("/d/") || url.path().contains("/product/"))
    {
        let current_host = cleaned.host_str().unwrap_or("").to_string();
        if let Some(stripped) = current_host.strip_prefix("www.") {
            cleaned
                .set_host(Some(stripped))
                .context("failed to strip www. from amazon host")?;
        }
        if let Some(caps) = AMAZON_PRODUCT_ID_RE.captures(url.path())
            && let Some(pid) = caps.get(1)
        {
            cleaned.set_path(&format!("/dp/{}", pid.as_str()));
        }
    }

    // --- Lenovo ---

    if host == "www.lenovo.com"
        && let Some(bundle) = params.get("bundleId")
    {
        cleaned.query_pairs_mut().append_pair("bundleId", bundle);
    }

    // --- Best Buy ---

    if host == "www.bestbuy.com"
        && url.path().contains(".p")
        && let Some(caps) = BESTBUY_SKU_RE.captures(url.path())
        && let Some(sku) = caps.get(1)
    {
        cleaned.set_path(&format!("/site/{}.p", sku.as_str()));
    }

    // --- Xiaohongshu ---

    if host == "www.xiaohongshu.com"
        && let Some(token) = params.get("xsec_token")
    {
        cleaned.query_pairs_mut().append_pair("xsec_token", token);
    }

    // --- Apple WeatherKit ---

    if host == "weatherkit.apple.com" {
        for key in ["lang", "party", "ids"] {
            if let Some(v) = params.get(key) {
                cleaned.query_pairs_mut().append_pair(key, v);
            }
        }
    }

    // --- Webtoons ---

    if host == "www.webtoons.com" {
        if let Some(title) = params.get("title_no") {
            cleaned.query_pairs_mut().append_pair("title_no", title);
        }
        if let Some(ep) = params.get("episode_no") {
            cleaned.query_pairs_mut().append_pair("episode_no", ep);
        }
    }

    // --- optional domain rewrites ---

    let cleaned_host = cleaned.host_str().unwrap_or("").to_lowercase();

    if settings.fix_twitter && (cleaned_host == "twitter.com" || cleaned_host == "x.com") {
        cleaned
            .set_host(Some("fxtwitter.com"))
            .context("failed to rewrite twitter host")?;
    }

    if settings.fix_bluesky && cleaned_host == "bsky.app" && url.path().contains("/post/") {
        cleaned
            .set_host(Some("fxbsky.app"))
            .context("failed to rewrite bluesky host")?;
    }

    Ok(cleaned.to_string())
}

fn extract_url(input: &str) -> Result<Url> {
    if let Ok(url) = Url::parse(input) {
        return Ok(url);
    }
    let found = URL_EXTRACTOR.find(input).context("no URL found in input")?;
    Url::parse(found.as_str()).with_context(|| format!("invalid extracted URL: {}", found.as_str()))
}

fn query_param(url: &Url, key: &str) -> Option<String> {
    url.query_pairs().find(|(k, _)| k == key).map(|(_, v)| v.into_owned())
}

fn is_amazon_host(host: &str) -> bool {
    let labels: Vec<&str> = host.split('.').collect();
    let Some(pos) = labels.iter().position(|&l| l == "amazon") else {
        return false;
    };
    let suffix = &labels[pos + 1..];
    if suffix.is_empty() || suffix.len() > 3 {
        return false;
    }
    suffix
        .iter()
        .all(|l| (1..=4).contains(&l.len()) && l.chars().all(|c| c.is_ascii_alphabetic()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cleans_to(input: &str, settings: &Settings, expected: &str) {
        let result = clean_link(input, settings).unwrap();
        assert_eq!(
            result, expected,
            "Mismatch for URL: {}\nExpected: {}\nGot:     {}",
            input, expected, result
        );
    }

    #[test]
    fn basic_cleaning() {
        let s = Settings::default();
        assert_cleans_to("https://example.com?utm_source=test", &s, "https://example.com/");
        assert_cleans_to(
            "https://example.com?utm_source=test&utm_medium=email",
            &s,
            "https://example.com/",
        );
        assert_cleans_to("https://example.com/path?foo=bar", &s, "https://example.com/path");
    }

    #[test]
    fn facebook_redirect() {
        let s = Settings::default();
        assert_cleans_to(
            "https://l.facebook.com/l.php?u=https%3A%2F%2Fexample.com%2Fpath%3Ffoo%3Dbar",
            &s,
            "https://example.com/path",
        );
    }

    #[test]
    fn href_li_redirect() {
        let s = Settings::default();
        assert_cleans_to(
            "https://href.li/?https://example.com/path?foo=bar",
            &s,
            "https://example.com/path",
        );
    }

    #[test]
    fn google_url_redirect() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.google.com/url?url=https://example.com/path?foo%3Dbar",
            &s,
            "https://example.com/path",
        );
    }

    #[test]
    fn youtube_preserve_video_id() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=test",
            &s,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        );
        assert_cleans_to(
            "https://youtube.com/watch?v=dQw4w9WgXcQ&foo=bar",
            &s,
            "https://youtube.com/watch?v=dQw4w9WgXcQ",
        );
    }

    #[test]
    fn youtube_shorten() {
        let s = Settings {
            youtube_shorten: true,
            ..Default::default()
        };
        assert_cleans_to(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=test",
            &s,
            "https://youtu.be/dQw4w9WgXcQ",
        );
        assert_cleans_to(
            "https://youtube.com/watch?v=dQw4w9WgXcQ",
            &s,
            "https://youtu.be/dQw4w9WgXcQ",
        );
    }

    #[test]
    fn youtube_preserve_timestamp() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s&utm_source=test",
            &s,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s",
        );
    }

    #[test]
    fn youtube_shorten_with_timestamp() {
        let s = Settings {
            youtube_shorten: true,
            ..Default::default()
        };
        assert_cleans_to(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s",
            &s,
            "https://youtu.be/dQw4w9WgXcQ?t=42s",
        );
    }

    #[test]
    fn youtube_playlist() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.youtube.com/playlist?list=PLsomeplaylist&utm_source=test",
            &s,
            "https://www.youtube.com/playlist?list=PLsomeplaylist",
        );
    }

    #[test]
    fn youtu_be_preserve_timestamp() {
        let s = Settings::default();
        assert_cleans_to(
            "https://youtu.be/dQw4w9WgXcQ?t=42s&utm_source=test",
            &s,
            "https://youtu.be/dQw4w9WgXcQ?t=42s",
        );
    }

    #[test]
    fn google_play_preserve_id() {
        let s = Settings::default();
        assert_cleans_to(
            "https://play.google.com/store/apps/details?id=com.example.app&utm_source=test",
            &s,
            "https://play.google.com/store/apps/details?id=com.example.app",
        );
    }

    #[test]
    fn macys_preserve_id() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.macys.com/shop/product/product-id?ID=12345&utm_source=test",
            &s,
            "https://www.macys.com/shop/product/product-id?ID=12345",
        );
    }

    #[test]
    fn facebook_story() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.facebook.com/story.php?story_fbid=12345&id=67890&utm_source=test",
            &s,
            "https://www.facebook.com/story.php?story_fbid=12345&id=67890",
        );
    }

    #[test]
    fn amazon_clean() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.amazon.com/dp/B08N5WRWNW/ref=something?foo=bar",
            &s,
            "https://amazon.com/dp/B08N5WRWNW",
        );
        assert_cleans_to(
            "https://www.amazon.com/product/B08N5WRWNW/something?foo=bar",
            &s,
            "https://amazon.com/dp/B08N5WRWNW",
        );
    }

    #[test]
    fn lenovo_bundle() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.lenovo.com/us/en/p/laptops/thinkpad/thinkpad-x1/x1-carbon-g9?bundleId=12345&utm_source=test",
            &s,
            "https://www.lenovo.com/us/en/p/laptops/thinkpad/thinkpad-x1/x1-carbon-g9?bundleId=12345",
        );
    }

    #[test]
    fn bestbuy_shorten() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.bestbuy.com/site/some-product/1234567.p?foo=bar",
            &s,
            "https://www.bestbuy.com/site/1234567.p",
        );
    }

    #[test]
    fn apple_weather() {
        let s = Settings::default();
        assert_cleans_to(
            "https://weatherkit.apple.com/api/v1/weather?lang=en&party=us&ids=12345&foo=bar",
            &s,
            "https://weatherkit.apple.com/api/v1/weather?lang=en&party=us&ids=12345",
        );
    }

    #[test]
    fn businesswire_redirect() {
        let s = Settings::default();
        assert_cleans_to(
            "https://cts.businesswire.com/ct/CT?id=smartlink&url=https%3A%2F%2Fexample.com%2Fpath",
            &s,
            "https://example.com/path",
        );
    }

    #[test]
    fn webtoons() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.webtoons.com/en/fantasy/some-series/list?title_no=1234&episode_no=56&foo=bar",
            &s,
            "https://www.webtoons.com/en/fantasy/some-series/list?title_no=1234&episode_no=56",
        );
    }

    #[test]
    fn google_search_preserve_q() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.google.com/search?q=rust+programming&utm_source=test",
            &s,
            "https://www.google.com/search?q=rust+programming",
        );
    }

    #[test]
    fn twitter_fix() {
        let s = Settings {
            fix_twitter: true,
            ..Default::default()
        };
        assert_cleans_to(
            "https://twitter.com/elonmusk/status/12345?utm_source=test",
            &s,
            "https://fxtwitter.com/elonmusk/status/12345",
        );
        assert_cleans_to(
            "https://x.com/elonmusk/status/12345?foo=bar",
            &s,
            "https://fxtwitter.com/elonmusk/status/12345",
        );
    }

    #[test]
    fn bluesky_fix() {
        let s = Settings {
            fix_bluesky: true,
            ..Default::default()
        };
        assert_cleans_to(
            "https://bsky.app/profile/did:plc:handle/post/12345?utm_source=test",
            &s,
            "https://fxbsky.app/profile/did:plc:handle/post/12345",
        );
    }

    #[test]
    fn url_embedded_in_text() {
        let s = Settings::default();
        assert_cleans_to(
            "Check this out: https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=test",
            &s,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        );
    }

    #[test]
    fn complex_amazon_yeti() {
        let s = Settings::default();
        assert_cleans_to(
            "https://www.amazon.com/YETI-Daytrip-Insulated-Cooler-Lunch/dp/B0F79VKFL8/?_encoding=UTF8&pd_rd_w=WhTdw&content-id=amzn1.sym.7e336f0f-97d5-42f6-9a1f-2f7317de4be6&pf_rd_p=7e336f0f-97d5-42f6-9a1f-2f7317de4be6&pf_rd_r=YFSZ1APYQ91M0EQBJPFG&pd_rd_wg=0td5E&pd_rd_r=2ae26c87-71ec-41df-968e-655ebd05d88d&th=1",
            &s,
            "https://amazon.com/dp/B0F79VKFL8",
        );
    }
}
