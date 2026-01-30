# Muusik

Muusik is a music manager/tagging software similar to [MusicBrainz Picard](https://picard.musicbrainz.org/).

Written with [Rust](https://www.rust-lang.org/), [TypeScript](https://www.typescriptlang.org/), [Tailwind CSS](https://tailwindcss.com/), and [Svelte](https://svelte.dev/).

## Features

- [x] Support for a majoirty of audio formats: MP3, M4A, AIFF, AAC, FLAC, OGG, WAV, and More
- [x] Scrape metadata from audio files 
- [x] Music tagging
- [x] Music art tagging
- [ ] Web interface
- [ ] Organize music files
- [ ] Create playlists
- [ ] Scrape metadata from multiple sources
- [ ] Scrape artwork from multiple sources

## Development

To develop Muusik, you need to have the following installed:

- [PNPM]([https://bun.sh/](https://pnpm.io/))
- [Rust](https://www.rust-lang.org/)
- [Sqlx CLI](https://crates.io/crates/sqlx-cli)
- [Sqlite](https://sqlite.org/)
- [OpenSSL](https://www.openssl.org/)

### Installing Dependencies


#### Arch Linux 
```bash
sudo pacman -S openssl sqlite
```

#### Fedora
```bash
sudo dnf install openssl-devel sqlite-devel
```

#### Debian/Ubuntu 
```bash
sudo apt install libssl-dev libsqlite3-dev
```

Install the the rest of the dependencies using the following commands:

```bash
# PNPM
curl -fsSL https://get.pnpm.io/install.sh | sh -
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Sqlx CLI
cargo install sqlx-cli
```

### Building/Running

```bash
cargo build --release # Or cargo run
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to use the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification when submitting commits.

## License

Muusik is licensed under the [GNU General Public License v3.0](https://choosealicense.com/licenses/gpl-3.0/).
