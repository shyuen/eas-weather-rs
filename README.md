# eas-weather-rs

A web microservice service that provides emergency alert system (EAS) information.

## Nix Environment
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

## Rust Workspace

### Running the Project
You generally run Rust projects using Cargo, Rust's package manager and build system. You can run the project with the following command:
```bash
cargo run
```

You can add more CLI arguments to your application as needed in the command line. For example, to see the help message for the application, you can run:
```bash
cargo run -- --help
```

### Unit Testing the Project
You can run the unit tests for the project using Cargo with the following command:
```
cargo test
```

### Priority Configuration Order
The configuration is loaded in the following order, with later values overriding earlier ones:
1. Default toml Configuration file (e.g., `default.toml`)
1. Configuration file specified by the `EAS_WEATHER_RS__APP__CONFIG_FILE` environment variable (e.g. `config.toml`)
1. Environment variable file (e.g., `.env`)
1. Environment variables (e.g., `EAS_WEATHER_RS__LOGGING__FORMAT`)
1. Command line arguments (e.g., `--log-format`)

### Environment Variables
All environment variables share the `EAS_WEATHER_RS` base prefix (derived from the package name `eas-weather-rs`), followed by a section, then the key — each part separated by `__`. This is the same naming shown in `cargo run -- --help` next to each flag (e.g. `[env: EAS_WEATHER_RS__WEBSERVER__PORT=]`). Any value may also be set via the `.env` file (see below).

The prefixes are:
- `EAS_WEATHER_RS__APP__` — application-level settings (e.g. `EAS_WEATHER_RS__APP__CONFIG_FILE`)
- `EAS_WEATHER_RS__LOGGING__` — logging (`EAS_WEATHER_RS__LOGGING__FORMAT`, `EAS_WEATHER_RS__LOGGING__TRACE_LEVEL`)
- `EAS_WEATHER_RS__WEBSERVER__` — web server (`EAS_WEATHER_RS__WEBSERVER__HOSTNAME`, `EAS_WEATHER_RS__WEBSERVER__PORT`, ...)
- `EAS_WEATHER_RS__DATABASE__` — database (`EAS_WEATHER_RS__DATABASE__CONN_URL_FILE`, `EAS_WEATHER_RS__DATABASE__CONN_MAX_RETRIES`, ...)

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

### Database Migrations

Migrations are managed by SQLx and live in the `migrations/` directory. Each file contains both `-- +migrate up` and `-- +migrate down` sections.

#### Building the Migration Binary
```bash
cargo build --bin eas-migrate
```

#### Running Migrations
The migration runner uses the same config precedence as the main server (CLI → env → file → defaults). It reads the database connection string from the configured source.

```bash
# Using default config (reads from ./config/mysql_conn_url)
cargo run --bin eas-migrate

# With explicit connection URL file
cargo run --bin eas-migrate -- --database-conn-url-file ./config/mysql_conn_url

# Via environment variable
EAS_WEATHER_RS__DATABASE__CONN_URL_FILE=./config/mysql_conn_url cargo run --bin eas-migrate
```

#### Running Rollbacks
To revert the last N migrations, use the SQLx CLI:
```bash
cargo install sqlx-cli
sqlx migrate revert --step N
```

#### Kubernetes Usage
In a k8s deployment, run `eas-migrate` as an **init container** before the main app container starts:

```yaml
initContainers:
  - name: migrate
    image: your-registry/eas-migrate:tag
    command: ["eas-migrate"]
    env:
      - name: EAS_WEATHER_RS__DATABASE__CONN_URL_FILE
        value: /etc/eas/conn_url
    volumeMounts:
      - name: db-credentials
        mountPath: /etc/eas
        readOnly: true
containers:
  - name: app
    image: your-registry/eas-weather-rs:tag
    # ...
```

### Logging
https://calmops.com/programming/rust/logging-and-distributed-tracing-in-rust-microservices/

### Coverage Report
Run the following command to generate an html report
```
cargo tarpaulin --out Html
```
