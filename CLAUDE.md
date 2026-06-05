# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`eas-weather-rs` is a web microservice (Rust, edition 2024) that serves emergency alert system (EAS) information. It exposes an HTTP API backed by a MySQL database.

## Commands

```bash
cargo run                          # Build and run the server (loads config, opens DB pool, starts HTTP server)
cargo run -- --help                # See all CLI flags (each mirrors a config field)
cargo build                        # Compile
cargo test                         # Run all unit tests
cargo test <name>                  # Run a single test by (sub)string match on its name
cargo test -- --nocapture          # Show stdout/log output during tests
cargo clippy                       # Lint
cargo fmt                          # Format
```

Nix users: `nix develop` (or `direnv` with an `.envrc` containing `use flake`) provisions the toolchain.

## Architecture: Hexagonal (Ports & Adapters)

The codebase is split into `src/lib/domain/` (business logic, dependency-free) and `src/lib/adaptors/` (concrete tech-specific implementations). `src/bin/server/main.rs` is the composition root that wires concrete adaptors into the generic services.

### Domain modules (`src/lib/domain/<context>/`)

Each bounded context (`alert`, `config`, `database`, `logging`, `meta`, `webserver`) follows the same layout:

- **`port.rs`** — a trait defining the interface the domain needs (e.g. `DatabasePort`, `LoggingPort`, `ConfigPort`). Ports use static dispatch (`impl Trait` / generic bounds), and async methods return `impl Future<...> + Send` rather than using `#[async_trait]`.
- **`service.rs`** — a generic struct parameterized by its port type, e.g. `DatabaseService<D: DatabasePort>`, `ConfigService<C: ConfigPort>`. Services hold a concrete port instance and delegate to it. They never name a concrete adaptor — only `main.rs` does.
- **`model.rs`** — the validated domain config struct for that context (e.g. `Database`, `Logging`, `Webserver`), assembled from newtypes.
- **`new_types/`** — one file per field, each a validated newtype wrapper (e.g. `DbConnectionString`, `WsPort`). They expose `new()` (validating, returns `Result`), `default()`, `get()`, and often a custom `Display`. Note: `DbConnectionString`'s `Display` masks credentials so connection strings can be logged safely — preserve this when touching logging.

A service is invoked across contexts by passing **other services** (not raw values), e.g. `DatabaseService::new(&conf_service, &log_service)`. The logging service/port is threaded through nearly every call so adaptors can emit structured logs at construction and connection time.

### Adaptors (`src/lib/adaptors/<tech>/`)

Each adaptor implements one or more domain ports:

- **`figment`** → `ConfigPort` (config loading)
- **`clap`** → CLI definition (`Cli` struct); its flags map 1:1 to config fields
- **`tracing`** → `LoggingPort` (structured logging via `tracing`/`tracing-subscriber`)
- **`xsqlx`** → `DatabasePort` + `DatabasePortAlert` (MySQL via `sqlx`)
- **`axum`** → `WebserverPort` (**active** HTTP server)
- **`poem`** → `WebserverPort` (alternative implementation, currently not wired in `main.rs` — `poem-openapi` provides Swagger UI)

To swap an implementation, change the concrete type in `main.rs` (e.g. `WebserverService<WebserverAxum>` vs `WebserverService<WebserverPoem>`); the service and domain code are unchanged. When adding a port method, update the trait in `domain/.../port.rs` first, then every adaptor that implements it.

### HTTP routing (axum)

Routes are defined in `src/lib/adaptors/axum/routes.rs`, nested under `/health` (`/startup`, `/readiness`, `/liveness`), `/meta` (`/conf`, `/raw_conf`), and `/test`. Handlers live in `src/lib/adaptors/axum/handlers/`. `AppState<MR, DR>` carries the meta and database ports into handlers.

## Configuration

Config is layered via `figment`, with **later sources overriding earlier** — i.e. effective priority is:

1. Code defaults (newtype `default()`)
2. `config/default.toml`
3. Selected config file (`config/config.toml`, or path from `--config-file` / `APP__CONFIG_FILE`)
4. `.env` file
5. Environment variables — prefixes: `APP__` (general), `LOGGING__`, `SERVER__`, `DATABASE__`, split on `__` into nested keys (e.g. `SERVER__PORT`)
6. CLI arguments (highest priority)

Secrets are loaded **from file paths**, not inline: the DB connection string comes from the file at `conn_url_file` (default `./config/mysql_conn_url`), and the API key / JWT key from their respective `*_file` paths. Config loading errors call `std::process::exit(1)` (planned exit) rather than panicking.

When adding a config field, update all of: the newtype in `domain/<ctx>/new_types/`, the `Config*` raw struct in `domain/config/model.rs`, the `Cli` struct in `adaptors/clap/model.rs`, and `config/default.toml`.
