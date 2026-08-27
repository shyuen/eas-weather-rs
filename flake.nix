{
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
          pname = "eas-weather-rs";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;
        };

        # Filter to only the server binary
        serverOnly = pkgs.runCommand "server-only" {} ''
          mkdir -p $out/bin
          cp ${myRustBuild}/bin/eas-weather-rs $out/bin/
        '';

        # Filter to only the migrate binary
        migrateOnly = pkgs.runCommand "migrate-only" {} ''
          mkdir -p $out/bin
          cp ${myRustBuild}/bin/eas-migrate $out/bin/
        '';

        # Server image
        serverImage = pkgs.dockerTools.buildLayeredImage {
          name = "eas-weather-rs";
          tag = "latest";
          contents = [ serverOnly ];
          extraCommands = ''
            mkdir -p home/eas
            echo "eas:x:1000:1000::/home/eas:/bin/sh" > etc/passwd
            echo "eas:x:1000:" > etc/group
          '';
          config = {
            User = "1000:1000";
            Cmd = [ "${serverOnly}/bin/eas-weather-rs" ];
            ExposedPorts = {
              "8080/tcp" = { };
            };
          };
        };

        # Migration runner image
        migrateImage = pkgs.dockerTools.buildLayeredImage {
          name = "eas-migrate";
          tag = "latest";
          contents = [ migrateOnly ];
          extraCommands = ''
            mkdir -p home/eas
            echo "eas:x:1000:1000::/home/eas:/bin/sh" > etc/passwd
            echo "eas:x:1000:" > etc/group
          '';
          config = {
            User = "1000:1000";
            Cmd = [ "${migrateOnly}/bin/eas-migrate" ];
          };
        };

      in {
        packages = {
          rustPackage = myRustBuild;
          docker = serverImage;
          server = serverImage;
          migrate = migrateImage;
        };

        # Build with: nix build .#docker (server) or nix build .#migrate
        # Load with: docker load < result
        defaultPackage = serverImage;

        # Development shell
        devShell = pkgs.mkShell {
          buildInputs =
            [ (rustVersion.override { extensions = [ "rust-src" "clippy" "rustfmt" ]; }) ];
          packages = with pkgs; [
            pkg-config
            openssl
            bacon
            cargo-nextest
            cargo-tarpaulin
            mariadb
          ];

          shellHook = ''
            MYSQL_DATA_DIR="$PWD/.mysql_data"
            MYSQL_SOCKET="$MYSQL_DATA_DIR/mysql.sock"
            if [ ! -d "$MYSQL_DATA_DIR" ]; then
              echo "Initialising MySQL data directory at $MYSQL_DATA_DIR..."
              mysql_install_db --datadir="$MYSQL_DATA_DIR" --auth-root-authentication-method=socket > /dev/null 2>&1
            fi
            echo ""
            echo "Start MySQL:"
            echo "  mysqld --datadir=$MYSQL_DATA_DIR --skip-grant-tables --socket=$MYSQL_SOCKET &"
            echo ""
            echo "Connect:"
            echo "  mysql -u root --socket=$MYSQL_SOCKET"
            echo ""
            echo "Create DB and configure connection URL:"
            echo "  mysql -u root --socket=$MYSQL_SOCKET -e \"CREATE DATABASE IF NOT EXISTS eas_weather;\""
            echo "  echo \"mysql://root@localhost/eas_weather\" > config/mysql_conn_url"
            echo ""
            echo "Connection URL: mysql://root@localhost/eas_weather"
          '';
        };
    });
}
