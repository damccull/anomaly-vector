{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem =
        {
          config,
          self',
          pkgs,
          lib,
          system,
          ...
        }:
        let
          runtimeDeps = with pkgs; [
          ];
          buildDeps = with pkgs; [
            clang
            lld
            lldb
            pkg-config
            rustPlatform.bindgenHook
          ];
          surrealdb-bin = pkgs.stdenv.mkDerivation rec {
            pname = "surrealdb";
            version = "3.0.5";
            src = pkgs.fetchurl {
              url = "https://github.com/surrealdb/surrealdb/releases/download/v${version}/surreal-v${version}.linux-amd64.tgz";
              hash = "sha256-SNvrpIlnZfM+B6zCUiQHPwhQwZCHIFK5F+vaH3tDdcs=";
            };
            sourceRoot = ".";
            nativeBuildInputs = [ pkgs.autoPatchelfHook ];
            buildInputs = [
              pkgs.stdenv.cc.cc.lib
              pkgs.openssl
            ];
            installPhase = ''
              mkdir -p $out/bin
              cp surreal $out/bin/
              chmod +x $out/bin/surreal
            '';
          };

          devDeps = with pkgs; [
            bacon
            bunyan-rs
            cargo-msrv
            cargo-nextest
            cargo-watch
            (cargo-whatfeatures.overrideAttrs (oldAttrs: rec {
              version = "0.9.13";
              src = fetchCrate {
                pname = "cargo-whatfeatures";
                version = "${version}";
                hash = "sha256-Nbyr7u47c6nImzYJvPVLfbqgDvzyXqR1C1tOLximuHU=";
              };

              cargoDeps = rustPlatform.fetchCargoVendor {
                inherit src;
                inherit (src) pname version;
                hash = "sha256-p95aYXsZM9xwP/OHEFwq4vRiXoO1n1M0X3TNbleH+Zw=";
              };
            }))
            clang
            fish
            flyctl
            gdb
            just
            lld
            lldb
            nushell
            panamax
            surrealdb-bin
          ];

          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          msrv = cargoToml.package.rust-version;

          rustPackage =
            features:
            (pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            }).buildRustPackage
              {
                inherit (cargoToml.package) name version;
                src = ./.;
                cargoLock.lockFile = ./Cargo.lock;
                buildFeatures = features;
                buildInputs = runtimeDeps;
                nativeBuildInputs = buildDeps;
                # Uncomment if your cargo tests require networking or otherwise
                # don't play nicely with the nix build sandbox:
                # doCheck = false;
              };

          mkDevShell =
            rustc:
            pkgs.mkShell {
              shellHook = ''
                export SHELL="${pkgs.fish}/bin/fish"

                # 1. [[ $- == *i* ]] checks if the CURRENT execution shell context is interactive.
                # 2. [ -z "$i" ] guards against some edge-case nested script evaluation loops.
                if [[ $- == *i* ]] && [ -z "$i" ]; then
                  export i=1
                  exec $SHELL -i
                fi
              '';
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
              LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
              buildInputs = runtimeDeps;
              nativeBuildInputs = buildDeps ++ devDeps ++ [ rustc ];
            };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
            config.allowUnfreePredicate =
              pkg:
              builtins.elem (lib.getName pkg) [
                "surrealdb"
              ];
          };

          packages.default = self'.packages.base;
          devShells.default = self'.devShells.stable;

          packages.base = (rustPackage "");
          packages.bunyan = (rustPackage "bunyan");
          packages.tokio-console = (rustPackage "tokio-console");

          devShells.nightly = (
            mkDevShell (
              pkgs.rust-bin.selectLatestNightlyWith (
                toolchain:
                toolchain.default.override {
                  extensions = [ "rust-analyzer" ];
                }
              )
            )
          );
          devShells.stable = (
            mkDevShell (
              pkgs.rust-bin.stable.latest.default.override {
                extensions = [ "rust-analyzer" ];
              }
            )
          );
          devShells.msrv = (
            mkDevShell (
              pkgs.rust-bin.stable.${msrv}.default.override {
                extensions = [ "rust-analyzer" ];
              }
            )
          );
        };
    };
}
