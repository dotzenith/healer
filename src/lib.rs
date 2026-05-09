// lc - A CLI tool that removes tracking parameters and junk from URLs.
// Copyright (C) 2024  lc contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// This project is based on Link Cleaner by Corbin Davenport, which is licensed
// under the GNU General Public License v3.0. The original source code can be
// found at https://github.com/corbindavenport/link-cleaner.

use regex::Regex;
use std::collections::HashMap;
use url::Url;

pub mod settings;

/// Settings controlling optional link cleaning behavior.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    /// Convert YouTube video links to the `youtu.be` short format.
    pub youtube_shorten: bool,
    /// Shorten Walmart product links to `walmart.com/ip/{id}`.
    pub walmart_shorten: bool,
    /// Replace `twitter.com` / `x.com` with `fxtwitter.com`.
    pub fix_twitter: bool,
    /// Replace `bsky.app` with `fxbsky.app`.
    pub fix_bluesky: bool,
    /// Amazon affiliate tracking ID appended to Amazon links.
    pub amazon_tracking_id: Option<String>,
}

/// Cleans a link according to the same rules as the JS Link Cleaner web app.
///
/// # Arguments
/// * `link`     – The raw URL string (may contain surrounding text).
/// * `settings` – Optional toggles (e.g. shorten YouTube, fix Twitter, etc.).
pub fn clean_link(link: &str, settings: &Settings) -> Result<String, String> {
    let mut old_link = parse_or_extract_url(link)?;

    // ------------------------------------------------------------------
    // 1. Fix known link shorteners / redirect services
    // ------------------------------------------------------------------
    if old_link.host_str() == Some("l.facebook.com") {
        if let Some(u) = get_param(&old_link, "u") {
            old_link = Url::parse(&u).map_err(|e| e.to_string())?;
        }
    } else if old_link.host_str() == Some("href.li") {
        // href.li puts the target URL in the query string without a key
        if let Some(q) = old_link.query() {
            old_link = Url::parse(q).map_err(|e| e.to_string())?;
        }
    } else if old_link.host_str() == Some("www.google.com") {
        if old_link.path() == "/url" {
            if let Some(url) = get_param(&old_link, "url") {
                old_link = Url::parse(&url).map_err(|e| e.to_string())?;
            }
        }
    }

    // ------------------------------------------------------------------
    // 2. Collect query parameters into a map for easy inspection
    // ------------------------------------------------------------------
    let params: HashMap<String, String> = old_link
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let host = old_link.host_str().unwrap_or("").to_lowercase();

    // ------------------------------------------------------------------
    // 3. Build the base cleaned link (origin + pathname only)
    // ------------------------------------------------------------------
    let origin = format!("{}://{}", old_link.scheme(), old_link.host_str().unwrap_or(""));
    let mut new_link = Url::parse(&format!("{}{}", origin, old_link.path()))
        .map_err(|e| e.to_string())?;

    // ------------------------------------------------------------------
    // 4. Always-preserved parameters (before optional transforms)
    // ------------------------------------------------------------------
    if let Some(q) = params.get("q") {
        new_link.query_pairs_mut().append_pair("q", q);
    }
    if host == "play.google.com" {
        if let Some(id) = params.get("id") {
            new_link.query_pairs_mut().append_pair("id", id);
        }
    }
    if host == "www.macys.com" {
        if let Some(id) = params.get("ID") {
            new_link.query_pairs_mut().append_pair("ID", id);
        }
    }

    // ------------------------------------------------------------------
    // 5. YouTube handling (must be checked before generic preserving)
    // ------------------------------------------------------------------
    let is_youtube = host.ends_with("youtube.com");
    let is_youtu_be = host == "youtu.be";

    if is_youtube && params.contains_key("v") {
        if settings.youtube_shorten {
            let re = Regex::new(r"^.*(youtu\.be/|embed/|shorts/|\?v=|\&v=)(?P<videoID>[^#&?]*).*")
                .unwrap();
            if let Some(caps) = re.captures(old_link.as_str()) {
                if let Some(video_id) = caps.name("videoID") {
                    new_link = Url::parse(&format!("https://youtu.be/{}", video_id.as_str()))
                        .map_err(|e| e.to_string())?;
                }
            }
        } else {
            if let Some(v) = params.get("v") {
                new_link.query_pairs_mut().append_pair("v", v);
            }
        }
        if let Some(t) = params.get("t") {
            new_link.query_pairs_mut().append_pair("t", t);
        }
    } else if is_youtube && old_link.path().contains("playlist") && params.contains_key("list") {
        if let Some(list) = params.get("list") {
            new_link.query_pairs_mut().append_pair("list", list);
        }
    } else if is_youtu_be && params.contains_key("t") {
        if let Some(t) = params.get("t") {
            new_link.query_pairs_mut().append_pair("t", t);
        }
    }

    // ------------------------------------------------------------------
    // 6. Other site-specific preserved parameters / path mutations
    // ------------------------------------------------------------------
    if host == "www.facebook.com" && old_link.path().contains("story.php") {
        if let Some(story) = params.get("story_fbid") {
            new_link.query_pairs_mut().append_pair("story_fbid", story);
        }
        if let Some(id) = params.get("id") {
            new_link.query_pairs_mut().append_pair("id", id);
        }
    }

    if host.contains("amazon")
        && (old_link.path().contains("/dp/")
            || old_link.path().contains("/d/")
            || old_link.path().contains("/product/"))
    {
        // Strip www. subdomain for Amazon
        let clean_host = new_link.host_str().unwrap_or("").replace("www.", "");
        if clean_host != new_link.host_str().unwrap_or("") {
            new_link
                .set_host(Some(&clean_host))
                .map_err(|e| format!("failed to set host: {}", e))?;
        }

        let re = Regex::new(r"(?:/dp/|/product/|/d/)(\w+|\d+)").unwrap();
        if let Some(caps) = re.captures(old_link.path()) {
            if let Some(pid) = caps.get(1) {
                new_link.set_path(&format!("/dp/{}", pid.as_str()));
            }
        }
    }

    if host == "www.lenovo.com" {
        if let Some(bundle) = params.get("bundleId") {
            new_link.query_pairs_mut().append_pair("bundleId", bundle);
        }
    }

    if host == "www.bestbuy.com" && old_link.path().contains(".p") {
        let re = Regex::new(r"/(\d+)\.p").unwrap();
        if let Some(caps) = re.captures(old_link.path()) {
            if let Some(pid) = caps.get(1) {
                new_link.set_path(&format!("/site/{}.p", pid.as_str()));
            }
        }
    }

    if host == "www.xiaohongshu.com" {
        if let Some(token) = params.get("xsec_token") {
            new_link.query_pairs_mut().append_pair("xsec_token", token);
        }
    }

    if host == "weatherkit.apple.com" {
        for key in ["lang", "party", "ids"] {
            if let Some(v) = params.get(key) {
                new_link.query_pairs_mut().append_pair(key, v);
            }
        }
    }

    if host == "cts.businesswire.com" {
        if let Some(url) = params.get("url") {
            new_link = Url::parse(url).map_err(|e| e.to_string())?;
        }
    }

    if host == "www.webtoons.com" {
        if let Some(title) = params.get("title_no") {
            new_link.query_pairs_mut().append_pair("title_no", title);
        }
        if let Some(ep) = params.get("episode_no") {
            new_link.query_pairs_mut().append_pair("episode_no", ep);
        }
    }

    // ------------------------------------------------------------------
    // 7. Optional feature-flag transforms
    // ------------------------------------------------------------------
    let new_host = new_link.host_str().unwrap_or("").to_lowercase();

    if settings.fix_twitter && (new_host == "twitter.com" || new_host == "x.com") {
        new_link
            .set_host(Some("fxtwitter.com"))
            .map_err(|e| format!("failed to set host: {}", e))?;
    }

    if settings.fix_bluesky
        && new_host == "bsky.app"
        && old_link.path().contains("/post/")
    {
        new_link
            .set_host(Some("fxbsky.app"))
            .map_err(|e| format!("failed to set host: {}", e))?;
    }

    if settings.walmart_shorten
        && new_host == "www.walmart.com"
        && old_link.path().contains("/ip/")
    {
        let re = Regex::new(r"/ip/.*/(\d+)").unwrap();
        if let Some(caps) = re.captures(old_link.path()) {
            if let Some(pid) = caps.get(1) {
                new_link.set_path(&format!("/ip/{}", pid.as_str()));
            }
        }
    }

    // ------------------------------------------------------------------
    // 8. Amazon affiliate tracking ID
    // ------------------------------------------------------------------
    if new_link.host_str().unwrap_or("").contains("amazon") {
        if let Some(ref tag) = settings.amazon_tracking_id {
            new_link.query_pairs_mut().append_pair("tag", tag);
        }
    }

    Ok(new_link.to_string())
}

/// Attempts to parse a URL. If that fails, tries to extract the first
/// `http(s)://...` substring and parse that instead.
fn parse_or_extract_url(link: &str) -> Result<Url, String> {
    match Url::parse(link) {
        Ok(u) => Ok(u),
        Err(_) => {
            let re = Regex::new(r"https?://\S+").unwrap();
            match re.find(link) {
                Some(m) => Url::parse(m.as_str()).map_err(|e| e.to_string()),
                None => Err("No valid URL found in the string.".to_string()),
            }
        }
    }
}

/// Helper: get a single query parameter by key.
fn get_param(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}
