// Pure-Rust compatibility tests based on the original JS Link Cleaner behavior.
// These expected outputs were derived by running the original JS cleanLink function
// through a Node.js test harness and recording the results.
//
// Original project: Link Cleaner by Corbin Davenport (GPL-3.0)
// https://github.com/corbindavenport/link-cleaner

use lc::Settings;

fn run_case(url: &str, settings: &Settings, expected: &str) {
    let result = lc::clean_link(url, settings).unwrap();
    assert_eq!(
        result, expected,
        "Mismatch for URL: {}\nExpected: {}\nGot: {}",
        url, expected, result
    );
}

#[test]
fn test_basic_cleaning() {
    let s = Settings::default();
    run_case(
        "https://example.com?utm_source=test",
        &s,
        "https://example.com/",
    );
    run_case(
        "https://example.com?utm_source=test&utm_medium=email",
        &s,
        "https://example.com/",
    );
    run_case("https://example.com/path?foo=bar", &s, "https://example.com/path");
}

#[test]
fn test_facebook_redirect() {
    let s = Settings::default();
    run_case(
        "https://l.facebook.com/l.php?u=https%3A%2F%2Fexample.com%2Fpath%3Ffoo%3Dbar",
        &s,
        "https://example.com/path",
    );
}

#[test]
fn test_href_li_redirect() {
    let s = Settings::default();
    run_case(
        "https://href.li/?https://example.com/path?foo=bar",
        &s,
        "https://example.com/path",
    );
}

#[test]
fn test_google_url_redirect() {
    let s = Settings::default();
    run_case(
        "https://www.google.com/url?url=https://example.com/path?foo%3Dbar",
        &s,
        "https://example.com/path",
    );
}

#[test]
fn test_youtube_preserve_video() {
    let s = Settings::default();
    run_case(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=test",
        &s,
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    );
    run_case(
        "https://youtube.com/watch?v=dQw4w9WgXcQ&foo=bar",
        &s,
        "https://youtube.com/watch?v=dQw4w9WgXcQ",
    );
}

#[test]
fn test_youtube_shorten() {
    let s = Settings {
        youtube_shorten: true,
        ..Default::default()
    };
    run_case(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=test",
        &s,
        "https://youtu.be/dQw4w9WgXcQ",
    );
    run_case(
        "https://youtube.com/watch?v=dQw4w9WgXcQ",
        &s,
        "https://youtu.be/dQw4w9WgXcQ",
    );
}

#[test]
fn test_youtube_preserve_timestamp() {
    let s = Settings::default();
    run_case(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s&utm_source=test",
        &s,
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s",
    );
}

#[test]
fn test_youtube_shorten_with_timestamp() {
    let s = Settings {
        youtube_shorten: true,
        ..Default::default()
    };
    run_case(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s",
        &s,
        "https://youtu.be/dQw4w9WgXcQ?t=42s",
    );
}

#[test]
fn test_youtube_playlist() {
    let s = Settings::default();
    run_case(
        "https://www.youtube.com/playlist?list=PLsomeplaylist&utm_source=test",
        &s,
        "https://www.youtube.com/playlist?list=PLsomeplaylist",
    );
}

#[test]
fn test_youtu_be_preserve_timestamp() {
    let s = Settings::default();
    run_case(
        "https://youtu.be/dQw4w9WgXcQ?t=42s&utm_source=test",
        &s,
        "https://youtu.be/dQw4w9WgXcQ?t=42s",
    );
}

#[test]
fn test_google_play_preserve_id() {
    let s = Settings::default();
    run_case(
        "https://play.google.com/store/apps/details?id=com.example.app&utm_source=test",
        &s,
        "https://play.google.com/store/apps/details?id=com.example.app",
    );
}

#[test]
fn test_macys_preserve_id() {
    let s = Settings::default();
    run_case(
        "https://www.macys.com/shop/product/product-id?ID=12345&utm_source=test",
        &s,
        "https://www.macys.com/shop/product/product-id?ID=12345",
    );
}

#[test]
fn test_facebook_story() {
    let s = Settings::default();
    run_case(
        "https://www.facebook.com/story.php?story_fbid=12345&id=67890&utm_source=test",
        &s,
        "https://www.facebook.com/story.php?story_fbid=12345&id=67890",
    );
}

#[test]
fn test_amazon_clean() {
    let s = Settings::default();
    run_case(
        "https://www.amazon.com/dp/B08N5WRWNW/ref=something?foo=bar",
        &s,
        "https://amazon.com/dp/B08N5WRWNW",
    );
    run_case(
        "https://www.amazon.com/product/B08N5WRWNW/something?foo=bar",
        &s,
        "https://amazon.com/dp/B08N5WRWNW",
    );
}

#[test]
fn test_lenovo_bundle() {
    let s = Settings::default();
    run_case(
        "https://www.lenovo.com/us/en/p/laptops/thinkpad/thinkpad-x1/x1-carbon-g9?bundleId=12345&utm_source=test",
        &s,
        "https://www.lenovo.com/us/en/p/laptops/thinkpad/thinkpad-x1/x1-carbon-g9?bundleId=12345",
    );
}

#[test]
fn test_bestbuy_shorten() {
    let s = Settings::default();
    run_case(
        "https://www.bestbuy.com/site/some-product/1234567.p?foo=bar",
        &s,
        "https://www.bestbuy.com/site/1234567.p",
    );
}

#[test]
fn test_apple_weather() {
    let s = Settings::default();
    run_case(
        "https://weatherkit.apple.com/api/v1/weather?lang=en&party=us&ids=12345&foo=bar",
        &s,
        "https://weatherkit.apple.com/api/v1/weather?lang=en&party=us&ids=12345",
    );
}

#[test]
fn test_businesswire_redirect() {
    let s = Settings::default();
    run_case(
        "https://cts.businesswire.com/ct/CT?id=smartlink&url=https%3A%2F%2Fexample.com%2Fpath",
        &s,
        "https://example.com/path",
    );
}

#[test]
fn test_webtoons() {
    let s = Settings::default();
    run_case(
        "https://www.webtoons.com/en/fantasy/some-series/list?title_no=1234&episode_no=56&foo=bar",
        &s,
        "https://www.webtoons.com/en/fantasy/some-series/list?title_no=1234&episode_no=56",
    );
}

#[test]
fn test_google_search_preserve_q() {
    let s = Settings::default();
    run_case(
        "https://www.google.com/search?q=rust+programming&utm_source=test",
        &s,
        "https://www.google.com/search?q=rust+programming",
    );
}

#[test]
fn test_twitter_fix() {
    let s = Settings {
        fix_twitter: true,
        ..Default::default()
    };
    run_case(
        "https://twitter.com/elonmusk/status/12345?utm_source=test",
        &s,
        "https://fxtwitter.com/elonmusk/status/12345",
    );
    run_case(
        "https://x.com/elonmusk/status/12345?foo=bar",
        &s,
        "https://fxtwitter.com/elonmusk/status/12345",
    );
}

#[test]
fn test_bluesky_fix() {
    let s = Settings {
        fix_bluesky: true,
        ..Default::default()
    };
    run_case(
        "https://bsky.app/profile/did:plc:handle/post/12345?utm_source=test",
        &s,
        "https://fxbsky.app/profile/did:plc:handle/post/12345",
    );
}

#[test]
fn test_walmart_shorten() {
    let s = Settings {
        walmart_shorten: true,
        ..Default::default()
    };
    run_case(
        "https://www.walmart.com/ip/some-product-name/13376108763?foo=bar",
        &s,
        "https://www.walmart.com/ip/13376108763",
    );
}

#[test]
fn test_amazon_affiliate() {
    let s = Settings {
        amazon_tracking_id: Some("myaffiliate-20".to_string()),
        ..Default::default()
    };
    run_case(
        "https://www.amazon.com/dp/B08N5WRWNW?foo=bar",
        &s,
        "https://amazon.com/dp/B08N5WRWNW?tag=myaffiliate-20",
    );
}

#[test]
fn test_url_in_text() {
    let s = Settings::default();
    run_case(
        "Check this out: https://www.youtube.com/watch?v=dQw4w9WgXcQ&utm_source=test",
        &s,
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    );
}

#[test]
fn test_complex_amazon_yeti() {
    let s = Settings::default();
    run_case(
        "https://www.amazon.com/YETI-Daytrip-Insulated-Cooler-Lunch/dp/B0F79VKFL8/?_encoding=UTF8&pd_rd_w=WhTdw&content-id=amzn1.sym.7e336f0f-97d5-42f6-9a1f-2f7317de4be6&pf_rd_p=7e336f0f-97d5-42f6-9a1f-2f7317de4be6&pf_rd_r=YFSZ1APYQ91M0EQBJPFG&pd_rd_wg=0td5E&pd_rd_r=2ae26c87-71ec-41df-968e-655ebd05d88d&th=1",
        &s,
        "https://amazon.com/dp/B0F79VKFL8",
    );
}
