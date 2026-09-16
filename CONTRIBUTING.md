# Contributing to eas-weather-rs

Thanks for considering a contribution. This project is small, so a few clear
guardrails keep it easy to review and safe to ship.

## Development environments — pick one

The repo supports two ways to develop:

1. **Plain `cargo`/`rustup`** (any OS, zero Nix) — the quickest path for a one-off
   change. Install Rust via [rustup.rs](https://rustup.rs).
2. **Nix flake** (recommended for anything deeper) — pin the exact toolchain and
   dependencies so your environment matches CI.

Both are first-class; pick whatever works for your platform.

### Platform-specific notes

| Platform | Recommendations |
|----------|-----------------|
| **Linux / macOS** | `nix develop` works natively. macOS on Apple Silicon is fully supported by the flake. |
| **Windows** | Nix only runs under **WSL2**. Install WSL2 + an Ubuntu distro, install Nix inside it, then `nix develop` there. Plain rustup on Windows also works — this is a portable Rust service. |
| **Containers** | `nix build .#docker-server` / `.#docker-migrate` builds the images; `docker load < result` loads them. |

### Nix hints

- The repo ships an `.envrc` (`use flake`) — with `direnv` installed the dev shell
  activates automatically on `cd`.
- `flake.lock` is machine-maintained (like `Cargo.lock`); you don't hand-edit it unless
  you're intentionally changing or updating the toolchain (`nix flake update`).
- If a build differs between `cargo` and `nix`, the discrepancy is a portability signal,
  not a flake bug. See README "What if a `nix build` behaves differently...".

## Working on the code

Trunk-based workflow: work on a short-lived branch, open a PR, `main` stays green.

### Required checks (what CI runs)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --no-deps -- -D warnings
cargo test -- --test-threads=1
```

The integration tests (`tests/db_integration.rs`) are `#[ignore]`-gated and exercise the
real HTTP → SQLx → MariaDB stack against a live DB. Run them with the bundled compose DB:

```bash
docker compose up -d db
EAS_WEATHER_RS_TEST_DB='mysql://root:root@localhost:3306/eas_weather' \
  cargo test --test db_integration -- --ignored --test-threads=1
```

CI additionally uploads a coverage report and (on `main` pushes) publishes the container
images and updates the GitOps repo.

### Adding a new binary

Binaries live under `src/bin/` and reuse the shared configuration/domain/service layers —
an EAS-related tool (a listener, a replayer, ...) is mostly CLI args + an outer adapter.
See README "Architecture" for the hexagonal conventions.

## Environment parity (read before bumping versions)

Rust is pinned in a few places; keep them consistent so "works on my machine" stays
"works in CI":

- `rust-toolchain.toml` → `channel = "1.91.0"` — consumed by **both** the Nix flake
  (`fromRustupToolchainFile`) and plain `rustup` developers, so the two environments agree
- `.github/workflows/ci.yml` → `RUST_TOOLCHAIN: 1.91.0`
- `Dockerfile` → `rust:1.91.0-bookworm` base (digest-pinned) + `rust-toolchain.toml`
  (rustup installs the pinned toolchain at build time); runtime stage is a
  digest-pinned `debian:bookworm-slim`

When bumping the toolchain, update them together and let CI prove the result. The
container images are built by CI from the pinned toolchain, so gating merge on green CI is
the only reliable cross-platform guarantee we have (machines differ; the pipeline doesn't).

## Deployment is separate

Serving the app (Helm chart, overlays, ArgoCD) lives in the
[`eas-weather-rs-k8s`](https://github.com/shyuen/eas-weather-rs-k8s) repo — changes there
go through its own PR + `verify` workflow. This repo's PRs only concern the code and its
images.
