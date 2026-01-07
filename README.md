# eas-weather-rs

A web microservice service that provides emergency alert system (EAS) information.

## NixOS Environment
*You may skip this section if you are not using NixOS.*

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
