# Healer

A CLI tool to remove tracking parameters and junk from URLs. Based on [Link Cleaner](https://github.com/corbindavenport/link-cleaner).

I liked the original project but it did not have a CLI that I could just use with my hotkey daemon.
So I used Kimi 2.6 to rewrite this as a rust CLI tool, so keep that in mind if you try to use this yourself.
This is not like any of my other work where I've hand-written (bad) code, so don't blame me for bugs.

## Build

#### Shell
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/dotzenith/healer/releases/latest/download/healer-installer.sh | sh
```

#### Powershell
```sh
powershell -ExecutionPolicy ByPass -c "irm https://github.com/dotzenith/healer/releases/latest/download/healer-installer.ps1 | iex"
```

#### Binaries
Pre-Compiled binaries for linux, mac, and windows are available in [Releases](https://github.com/dotzenith/healer/releases)


#### Source
```sh
git clone https://github.com/dotzenith/healer
cd healer
cargo build --release
./target/release/hl
```

## Usage

```
Clean tracking parameters and junk from URLs.

Usage: hl [OPTIONS] <URL>

Arguments:
  <URL>  The URL to clean

Options:
      --youtube-shorten  Shorten YouTube video links to youtu.be format
      --fix-twitter      Replace twitter.com / x.com with fxtwitter.com
      --fix-bluesky      Replace bsky.app with fxbsky.app
      --fix-instagram    Replace instagram.com with vxinstagram.com
      --clipboard        Copy the cleaned URL to the system clipboard
      --verbose          Print verbose debugging output (old link, settings, new link)
      --config <PATH>    Path to a custom configuration file
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

#### Generic Link
```sh
hl "https://example.com?utm_source=test"
# → https://example.com/
```

#### Shorten YouTube links
```sh
hl "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --youtube-shorten
# → https://youtu.be/dQw4w9WgXcQ
```

#### Copy straight to clipboard
```sh
hl "https://example.com?foo=bar" --clipboard
```

## Configuration

`hl` reads an optional config file at `~/.config/hl/config.toml`. CLI flags override config values.

### Example `config.toml`

```toml
youtube_shorten = true
fix_twitter = false
fix_bluesky = false
fix_instagram = false
clipboard = true
```

## License

GPL-3.0
