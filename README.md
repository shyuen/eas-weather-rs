# eas-weather-rs

eas-weather-rs is a Rust-based microservice that provides emergency alert system (EAS) information via an HTTP API.

Built with a hexagonal architecture, it features SQLx database migrations, OpenAPI (Swagger) integration for API documentation, and follows a domain-driven design with clear separation of concerns, utilizing the new type pattern for domain modeling.

This project was created to help learn Rust and the hexagonal pattern, originally started without AI assistance as an opportunity to learn to use AI effectively for coding. 

## General Setup Information

### Nix Development Environment
*You may skip this section if you are not using Nix.*

This project includes a `flake.nix` file for setting up the development environment on a NixOS machine.
You can enter the development environment by running the following at the location of the `flake.nix` file.:
```
nix develop
```
### Direnv Integration
You can bootstrap initiatlizing the developement environment by create an `.envrc` file with the following content:
```bash
use flake
```
This will require `direnv` to be installed and enabled in your shell.

### Rust Workspace



#### Unit Testing Rust Projects
You can run the unit tests for the project using Cargo with the following command:
```
cargo test
```

## Project Specific Information

### Priority Configuration Order
The configuration is loaded in the following order, with later values overriding earlier ones:
1. Default toml Configuration file (e.g., `default.toml`)
1. Configuration file specified by the `EAS_WEATHER_RS__APP__CONFIG_FILE` environment variable (e.g. `config.toml`)
1. Environment variable file (e.g., `.env`)
1. Environment variables (e.g., `EAS_WEATHER_RS__LOGGING__FORMAT`)
1. Command line arguments (e.g., `--log-format`)

### Environment Variables
All environment variables share the `EAS_WEATHER_RS` base prefix (derived from the package name `eas-weather-rs`), followed by a section, then the key — each part separated by `__`. This is the same naming shown in `cargo run -- --help` next to each flag (e.g. `[env: EAS_WEATHER_RS__WEBSERVER__PORT=]`). Any value may also be set via the `.env` file (see below).

To see all available environment variables with examples and descriptions, please reference the `.env.example` file in the project root. This file contains commented-out examples for every configurable option.

The prefixes are:
- `EAS_WEATHER_RS__APP__` — application-level settings (e.g. `EAS_WEATHER_RS__APP__CONFIG_FILE`)
- `EAS_WEATHER_RS__LOGGING__` — logging (`EAS_WEATHER_RS__LOGGING__FORMAT`, `EAS_WEATHER_RS__LOGGING__TRACE_LEVEL`)
- `EAS_WEATHER_RS__WEBSERVER__` — web server (`EAS_WEATHER_RS__WEBSERVER__HOSTNAME`, `EAS_WEATHER_RS__WEBSERVER__PORT`, ...)
- `EAS_WEATHER_RS__DATABASE__` — database (`EAS_WEATHER_RS__DATABASE__CONN_URL_FILE`, `EAS_WEATHER_RS__DATABASE__CONN_MAX_RETRIES`, ...)

### Architectural Benefits

This project follows a hexagonal (ports and adapters) architecture which provides several benefits:

**Dependency Swapping**: External dependencies (database, web framework) can be changed without modifying domain logic. For example, switching from Axum to Poem or MySQL to PostgreSQL only requires changing adaptor implementations.

**Testability**: Domain services depend only on ports (traits), making them easy to test with mock implementations.

**Shared Logic Without Versioning**: Core domain logic (validation, workflows, modeling) is versioned together with the application. When adding new binaries (like a migration tool or future listener), they automatically get the latest domain logic without needing to manage separate library versions.

**Safe Binary Addition**: New EAS-related tools can be added as separate binaries under `src/bin/` while reusing:
- Identical configuration loading patterns
- Identical logging setup
- Identical service construction
- Identical domain validation logic
Only the outermost layer (CLI args, socket listener, HTTP server) and app-specific business logic differ.

Example:
```bash
export EAS_WEATHER_RS__WEBSERVER__PORT=8080
export EAS_WEATHER_RS__LOGGING__FORMAT="json"
```

**Secrets are paths, never inline values:** database connection string, API key, and JWT key come from the files referenced by their respective `*_file` env vars / config keys (e.g. `EAS_WEATHER_RS__DATABASE__CONN_URL_FILE`), and are masked in logs.

**.env file:** a `.env` file in the project root can set any of these variables. It uses the same keys without the leading `export`:
```
EAS_WEATHER_RS__WEBSERVER__HOSTNAME="localhost"
EAS_WEATHER_RS__LOGGING__FORMAT="json"
```

The defaults for every key are defined in `config/default.toml`.

### Running the Main Application
You can run the main eas-weather-rs-server application using Cargo:
```bash
cargo run --bin eas-weather-rs-server
```

This starts the `eas-weather-rs-server` binary, which serves EAS alert data via HTTP. The application will load configuration according to the priority order described above.

You can add CLI arguments to customize the runtime behavior. For example, to see all available options:
```bash
cargo run --bin eas-weather-rs-server --help
```

Common usage patterns include:
- Using default configuration (loads from `config/default.toml`): `cargo run`
- Specifying a custom config file: `cargo run -- --config-file ./config/custom.toml`
- Overriding specific settings via environment variables: `EAS_WEATHER_RS__WEBSERVER__PORT=3000 cargo run`
- Using a .env file: create a `.env` file in the project root and run `cargo run`

### Database Migrations

Migrations are managed by SQLx and live in the `migrations/` directory. Each migration consists of two separate files: `{timestamp}_name.up.sql` for the migration and `{timestamp}_name.down.sql` for the rollback. When creating new migrations, use `sqlx migrate add --reversible` (or `-r`) to generate both files.

#### Building the Migration Binary
```bash
cargo build --bin eas-weather-rs-migrate
```

#### Running Migrations
The migration runner uses the same config precedence as the main server (CLI → env → file → defaults). It reads the database connection string from the configured source.

```bash
# Using default config (reads from ./config/mysql_conn_url)
cargo run --bin eas-weather-rs-migrate

# With explicit connection URL file
cargo run --bin eas-weather-rs-migrate -- --database-conn-url-file ./config/mysql_conn_url

# Via environment variable
EAS_WEATHER_RS__DATABASE__CONN_URL_FILE=./config/mysql_conn_url cargo run --bin eas-weather-rs-migrate
```

#### Running Rollbacks
To revert the last N migrations, use the SQLx CLI:
```bash
cargo install sqlx-cli
sqlx migrate revert --step N
```

#### Kubernetes Usage
In a k8s deployment, run `eas-weather-rs-migrate` as an **init container** before the main app container starts:

```yaml
initContainers:
  - name: migrate
    image: your-registry/eas-weather-rs-migrate:tag
    command: ["eas-weather-rs-migrate"]
    env:
      - name: EAS_WEATHER_RS__DATABASE__CONN_URL_FILE
        value: /etc/eas/conn_url
    volumeMounts:
      - name: db-credentials
        mountPath: /etc/eas
        readOnly: true
containers:
  - name: app
    image: your-registry/eas-weather-rs-server:tag
    # ...
```

### Logging
https://calmops.com/programming/rust/logging-and-distributed-tracing-in-rust-microservices/

### Container Images

#### Using Docker (Dockerfile)
```bash
# Build server image
docker build -t eas-weather-rs-server .

# Build migration image
docker build -t eas-weather-rs-migrate .
```

#### Using Nix (flake)
```bash
# Build server image
nix build .#docker
docker load < result

# Build migration image
nix build .#migrate
docker load < result
```

#### Running Containers
```bash
# Server
docker run -p 8080:8080 eas-weather-rs-server

# Migrations (run as init container or manually)
docker run --entrypoint /app/eas-weather-rs-migrate eas-weather-rs-server
```

### Coverage Report
Run the following command to generate an html report
```
cargo tarpaulin --out Html
```
