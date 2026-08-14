# AGENTS.md — eas-weather-rs

Rust 2024 edition web microservice (single crate, bin+lib) serving EAS alert data via HTTP/MySQL.

## Commands

| Command | Purpose |
|---------|---------|
| `cargo run` | Build & run server |
| `cargo test` / `cargo test <name>` | All / single test (substring match) |
| `cargo clippy` / `cargo fmt` | Lint / format |
| `nix develop` | Dev shell (bacon, cargo-nextest, clippy, rustfmt) |

## Architecture

**Hexagonal (Ports & Adapters).** Domain in `src/lib/domain/<context>/` is dependency-free; adaptors in `src/lib/adaptors/<tech>/` implement domain traits.

Each domain context: `port.rs` (trait), `service.rs` (generic struct), `model.rs` (validated config), `new_types/` (field wrappers). Contexts: `alert`, `config`, `database`, `logging`, `meta`, `webserver`, `utils`.

`src/bin/server/main.rs` wires concrete adaptors into generic services. Services receive **other services** (not raw values). Logging port threaded through nearly every constructor.

## Config loading (figment)

Priority (later overrides): code defaults → `config/default.toml` → selected config file (`--config-file`) → `.env` → env vars → CLI args.

Env prefixes: `EAS_WEATHER_RS__APP__`, `EAS_WEATHER_RS__LOGGING__`, `EAS_WEATHER_RS__SERVER__`, `EAS_WEATHER_RS__DATABASE__`. The base prefix `EAS_WEATHER_RS` is derived from the crate name (hyphens → underscores) and verified at compile time in `src/lib/adaptors/figment/model.rs`. Env vars split on `__` → nested keys (e.g. `EAS_WEATHER_RS__SERVER__PORT` → `server.port`). clap's `#[arg(env = "...")]` attributes are written as literals (clap can't take a const).

**Two-phase load:** figment loads config twice — first to extract `config_file` path, second with all sources.

Secrets come from `*_file` paths, never inline. `DbConnectionString::Display` masks credentials for safe logging.

## Gotchas

- `DatabaseService<D>` requires `D: DatabasePort + AlertPort` (double bound on the same type param)
- Port traits use **static dispatch** (`impl Future<...> + Send`), never `#[async_trait]`
- Planned exits: `std::process::exit(1)` everywhere, never panics
- `.gitignore` lists `conf/config.toml` (old path), but the actual directory is `config/`
- `poem` adaptor exists with swagger-ui but is **not wired** in `main.rs` — axum is the active webserver
- Adding a config field: update newtype → `Config*` raw struct (`domain/config/model.rs`) → `Cli` struct → `config/default.toml`
- `test_api` file contains a sample API key (`MOHmohMoh`)
