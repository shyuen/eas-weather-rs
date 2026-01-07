{
  # https://johns.codes/blog/rust-enviorment-and-docker-build-with-nix-flakes
  description = "A flake for building a Rust project and its Docker image";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        myRustBuild = rustPlatform.buildRustPackage {
          pname = "eas-weather-rs"; # make this what ever your cargo.toml package.name is
          version = "0.1.0";
          src = ./.; # the folder with the cargo.toml

          cargoLock.lockFile = ./Cargo.lock;
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "eas-weather-rs";
          tag = "latest";
          config = {
            Cmd = [ "${myRustBuild}/bin/eas-weather-rs" ];
            ExposedPorts = {
              "8080/tcp" = { };
            };
          };
        };

    in {
      packages = {
        rustPackage = myRustBuild;
        docker = dockerImage;
      };

      # Build Docker Image using nix
      # Build with `nix build .#docker` in the directory containing this flake
      # ls -lh result
      # docker load < result
      # docker run <container_image_name>
      # docker run --net=host eas-weather-rs
      defaultPackage = dockerImage;

      docker = dockerImage;

      # Development Shell
      # Run `nix develop` in the directory containing this flake.nix
      devShell = pkgs.mkShell {
        buildInputs =
          [ (rustVersion.override { extensions = [ "rust-src"]; }) ];
        packages = with pkgs; [
          pkg-config
          openssl
          bacon
          cargo-nextest
          clippy
          rustfmt
        ];
        # Set environment variables
        # env.LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [ # Shared library path on Linux
        #   stdenv.cc.cc.lib
        #   pkgs.libz
        # ];
      };
  });
}









# {
#   description = "Flake shell for Rust projects";
#   # Reference https://youtu.be/6fftiTJ2vuQ
#   inputs = {
#     # Nixpkgs stable
#     # nixpkgs-stable.url = "github:NixOS/nixpkgs?ref=nixos-24.11";

#     # Nixpkgs unstable
#     # nixpkgs-unstable.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

#     nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

#   };
#   outputs = { self, nixpkgs,... }:
#       let
#         appName = "myapp";
#         pkgs = nixpkgs.legacyPackages."x86_64-linux";

#         # forAllSystems = function:
#         #   nixpkgs.lib.genAttrs [
#         #   "x86_64-linux"
#         #   "aarch64-linux"
#         #   ] (system:
#         #     function (import nixpkgs {
#         #       inherit system;
#         #       config.allowUnfree = true;
#         #       overlays = [
#         #         #inputs.something.overlays.default
#         #       ];
#         #     }));

#       in {

#       # buildApp = rustPlatform.buildRustPackage {
#       #   pname = appName;
#       #   version = "0.1.0";
#       #   src = ./.;
#       #   cargoLock.lockFile = ./Cargo.lock;
#       # };

#       # Build Docker Image using nix
#       # Build with `nix build .#docker` in the directory containing this flake
#       # ls -lh result
#       # docker load < result
#       # docker run <container_image_name>
#       baselayer = pkgs.dockerTools.buildLayeredImage {
#         name = appName;
#         tag = "latest";
#         #created = "now"; # Optional: set creation time, using it will make the build non-reproducible

#         # Include packages in the image
#         contents = with pkgs; [
#           rustc
#           cargo
#           gcc
#           pkg-config
#           openssl
#         ];

#         # Configure the container
#         config = {
#           Cmd = [ "hello" ];
#           ExposedPorts = {
#             "8080/tcp" = { };
#           };
#         };

#       };

#       buildLayer = pkgs.dockerTools.buildLayeredImage {
#         name = "layered";
#         tag = "latest";
#         fromImage = self.baselayer;
#         contents = [ pkgs.curl ];
#         config = {
#           Cmd = [ "curl" "--help" ];
#         };
#       };

#       docker = pkgs.dockerTools.buildImage {
#         name = "layered-on-top";
#         tag = "latest";
#         fromImage = self.buildLayer;
#         contents = [ pkgs.curl ];
#         config = {
#           Cmd = [ "curl" "--help" ];
#         };
#       };

#       # Development Shell
#       # Run `nix develop` in the directory containing this flake.nix
#       devShells.x86_64-linux.default = pkgs.mkShell {
#         # Add required packages
#         packages = with pkgs; [
#           rustc
#           cargo
#           gcc
#           pkg-config
#           openssl
#         ];
#         # Set environment variables
#         env.LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [ # Shared library path on Linux
#           stdenv.cc.cc.lib
#           pkgs.libz
#         ];
#       };
#     };
# }
