use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::Parser;
use healer::settings::{ConfigFile, default_config_path};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hl")]
#[command(about = "Clean tracking parameters and junk from URLs.")]
#[command(version)]
struct Cli {
    /// The URL to clean.
    url: String,

    /// Shorten YouTube video links to youtu.be format.
    #[arg(long)]
    youtube_short: bool,

    /// Replace twitter.com / x.com with fxtwitter.com.
    #[arg(long)]
    fix_twitter: bool,

    /// Replace bsky.app with fxbsky.app.
    #[arg(long)]
    fix_bluesky: bool,

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

    let config_path = cli
        .config
        .clone()
        .or_else(default_config_path)
        .context("could not determine config directory — use --config to specify one")?;

    let config = ConfigFile::load(&config_path)?;
    let mut settings = config.to_settings();

    let use_clipboard = cli.clipboard || config.clipboard.unwrap_or(false);

    if cli.youtube_short {
        settings.youtube_shorten = true;
    }
    if cli.fix_twitter {
        settings.fix_twitter = true;
    }
    if cli.fix_bluesky {
        settings.fix_bluesky = true;
    }

    if cli.verbose {
        eprintln!("Old link:    {}", cli.url);
        eprintln!("Settings:    {:?}", settings);
        eprintln!("Config file: {}", config_path.display());
    }

    let cleaned =
        healer::clean_link(&cli.url, &settings).with_context(|| format!("failed to clean URL: {}", cli.url))?;

    if use_clipboard {
        let mut clipboard = Clipboard::new().context("failed to access system clipboard")?;
        clipboard
            .set_text(&cleaned)
            .context("failed to copy cleaned URL to clipboard")?;
        println!("{cleaned} [Copied to clipboard]");
    } else {
        println!("{cleaned}");
    }

    Ok(())
}
