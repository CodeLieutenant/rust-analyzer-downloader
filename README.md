# Rust Analyzer Downloader

### Motivation

1. I always wanted to have rust-analyzer in my $HOME/bin folder, from where I could share it with VSCode and NeoVIM (possibly others if I find them in the future). This was done using bash script without any error handling, which failed from time to time, also I wanted to use Rust for some CLI project and have a little bit more practice with the language and ecosystem and this provided perfect opportunity.

2. Also bash script was not portable to Windows, since i do most of my rust development on Windows PC, this was one deal breaker for me to rewrite the script into Rust.

### Usage

```
Downloads and gets versions for Rust Analyzer

USAGE:
    rust-analyzer-downloader <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    download
    get-versions
    help            Print this message or the help of the given subcommand(s)
```

### Building

```
cargo build --release
```
