{
  description = "basic rust template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "utils/flake-utils";
      };
    };
  };

  outputs = { self, utils, ... }@inputs:
    utils.lib.mkFlake {
      inherit self inputs;
      channels.nixpkgs.overlaysBuilder = channels:
        [ inputs.rust-overlay.overlays.rust-overlay ];

      outputsBuilder = channels:
        let
          pkgs = channels.nixpkgs;
          toolchain =
            pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          platform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
          cargoMeta =
            (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
        in {
          packages = rec {
            ical2org = platform.buildRustPackage {
              pname = "ical2org";
              inherit (cargoMeta) version;

              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              nativeBuildInputs =
                [ toolchain platform.cargoBuildHook platform.cargoCheckHook ];

              meta = {
                inherit (cargoMeta) description homepage;
                license = [ pkgs.lib.licenses.gpl3Plus ];
              };
            };
            default = ical2org;
          };

          devShells.default = channels.nixpkgs.mkShell {
            RUST_LOG = "debug";
            RUST_BACKTRACE = 1;
            buildInputs = [
              toolchain
              pkgs.rust-analyzer
              pkgs.cargo-audit
              pkgs.cargo-license
            ];
          };
        };
    };
}
