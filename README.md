# lc

A CLI tool that removes tracking parameters and junk from URLs. Based on [Link Cleaner](https://github.com/corbindavenport/link-cleaner) by Corbin Davenport.

I liked the original project but it did not have a CLI that I could just use with my hotkey daemon.
So I used Kimi 2.6 to rewrite this as a rust CLI tool, so keep that in mind if you try to use this yourself.
This is not like any of my other work where I've hand-written (bad) code, so don't blame me for bugs.

## Build

```sh
git clone https://github.com/dotzenith/cleaner
cd cleaner
cargo build --release
```

The binary is at `target/release/lc`.

## Usage

```
Clean tracking parameters and junk from URLs.

Usage: lc [OPTIONS] <URL>

Arguments:
  <URL>  The URL to clean

Options:
      --youtube-short    Shorten YouTube video links to youtu.be format
      --walmart-short    Shorten Walmart product links to walmart.com/ip/{id} format
      --fix-twitter      Replace twitter.com / x.com with fxtwitter.com
      --fix-bluesky      Replace bsky.app with fxbsky.app
      --amazon-tag <ID>  Add an Amazon affiliate tracking ID to Amazon links
      --clipboard        Copy the cleaned URL to the system clipboard
      --verbose          Print verbose debugging output (old link, settings, new link)
      --config <PATH>    Path to a custom configuration file
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

```sh
# Basic cleaning
lc "https://example.com?utm_source=test"
# → https://example.com/

# Shorten YouTube links
lc "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --youtube-short
# → https://youtu.be/dQw4w9WgXcQ

# Copy result to clipboard
lc "https://example.com?foo=bar" --clipboard

# Use a custom config file
lc "https://example.com" --config /path/to/config.toml
```

## Configuration

`lc` reads an optional config file at `~/.config/lc/config.toml`. CLI flags override config values.

### Example `config.toml`

```toml
youtube_shorten = true
walmart_shorten = true
fix_twitter = false
fix_bluesky = false
amazon_tracking_id = "mytag-20"
```

## License

GPL-3.0
