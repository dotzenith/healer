// lc - A CLI tool that removes tracking parameters and junk from URLs.
// Copyright (C) 2024  lc contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use lc::settings::{default_config_path, ConfigFile};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "lc")]
#[command(about = "Clean tracking parameters and junk from URLs.")]
#[command(version)]
struct Cli {
    /// The URL to clean.
    url: String,

    /// Shorten YouTube video links to youtu.be format.
    #[arg(long)]
    youtube_short: bool,

    /// Shorten Walmart product links to walmart.com/ip/{id} format.
    #[arg(long)]
    walmart_short: bool,

    /// Replace twitter.com / x.com with fxtwitter.com.
    #[arg(long)]
    fix_twitter: bool,

    /// Replace bsky.app with fxbsky.app.
    #[arg(long)]
    fix_bluesky: bool,

    /// Add an Amazon affiliate tracking ID to Amazon links.
    #[arg(long, value_name = "ID")]
    amazon_tag: Option<String>,

    /// Copy the cleaned URL to the system clipboard.
    #[arg(long)]
    clipboard: bool,

    /// Print verbose debugging output (old link, settings, new link).
    #[arg(long)]
    verbose: bool,

    /// Path to a custom configuration file.
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // ------------------------------------------------------------------
    // 1. Load config file (or default empty config)
    // ------------------------------------------------------------------
    let config_path = cli
        .config
        .clone()
        .or_else(default_config_path)
        .context("Could not determine config directory. Please specify --config.")?;

    let config = ConfigFile::load(&config_path)?;
    let mut settings = config.to_settings();

    // ------------------------------------------------------------------
    // 2. Override with CLI flags
    // ------------------------------------------------------------------
    if cli.youtube_short {
        settings.youtube_shorten = true;
    }
    if cli.walmart_short {
        settings.walmart_shorten = true;
    }
    if cli.fix_twitter {
        settings.fix_twitter = true;
    }
    if cli.fix_bluesky {
        settings.fix_bluesky = true;
    }
    if cli.amazon_tag.is_some() {
        settings.amazon_tracking_id = cli.amazon_tag;
    }

    // ------------------------------------------------------------------
    // 3. Verbose output (optional)
    // ------------------------------------------------------------------
    if cli.verbose {
        eprintln!("Old link:    {}", cli.url);
        eprintln!("Settings:    {:?}", settings);
        eprintln!("Config file: {}", config_path.display());
    }

    // ------------------------------------------------------------------
    // 4. Clean the link
    // ------------------------------------------------------------------
    let cleaned = lc::clean_link(&cli.url, &settings)
        .map_err(|e| anyhow::anyhow!("Failed to clean URL: {} - {}", cli.url, e))?;

    if cli.verbose {
        eprintln!("New link:    {}", cleaned);
    }

    // ------------------------------------------------------------------
    // 5. Output
    // ------------------------------------------------------------------
    println!("{}", cleaned);

    if cli.clipboard {
        Clipboard::new()
            .with_context(|| "Failed to access system clipboard.")?
            .set_text(&cleaned)
            .with_context(|| "Failed to copy cleaned URL to clipboard.")?;
        eprintln!("Copied to clipboard.");
    }

    Ok(())
}
