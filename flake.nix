{
  description = "basic rust template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "utils/flake-utils";
      };
    };
  };

  outputs =
    { self, utils, ... }@inputs:
    utils.lib.mkFlake {
      inherit self inputs;
      channels.nixpkgs.overlaysBuilder = channels: [ inputs.rust-overlay.overlays.rust-overlay ];

      outputsBuilder =
        channels:
        let
          pkgs = channels.nixpkgs;
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          platform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
          cargoMeta = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
        in
        rec {
          packages = rec {
            ical2org = platform.buildRustPackage {
              pname = "ical2org";
              inherit (cargoMeta) version;

              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              nativeBuildInputs = [
                toolchain
                platform.cargoBuildHook
                platform.cargoCheckHook
              ];

              meta = {
                inherit (cargoMeta) description homepage;
                license = [ pkgs.lib.licenses.gpl3Plus ];
              };
            };
            default = ical2org;
          };

          checks =
            let
              runCargo' =
                name: env: buildCommand:
                pkgs.stdenv.mkDerivation (
                  {
                    preferLocalBuild = true;
                    allowSubstitutes = false;

                    RUSTFLAGS = "-Dwarnings";
                    RUSTDOCFLAGS = "-Dwarnings";

                    src = ./.;
                    cargoDeps = platform.importCargoLock { lockFile = ./Cargo.lock; };

                    nativeBuildInputs =
                      [ toolchain ]
                      ++ (with platform; [
                        cargoSetupHook
                        pkgs.python3
                      ]);

                    inherit name;

                    buildPhase = ''
                      runHook preBuild
                      mkdir $out
                      ${buildCommand}
                      runHook postBuild
                    '';
                  }
                  // env
                );
              runCargo = name: runCargo' name { };
            in
            {
              inherit (packages) ical2org;

              devshell = devShells.default;

              clippy = runCargo "ical2org-check-clippy" ''
                cargo clippy --all-targets
              '';

              doc = runCargo "ical2org-check-docs" ''
                cargo doc --workspace
              '';

              fmt = runCargo "ical2org-check-formatting" ''
                cargo fmt --all -- --check
              '';

              test = runCargo "ical2org-check-tests" ''
                cargo test
              '';
            };

          devShells.default = channels.nixpkgs.mkShell {
            RUST_LOG = "debug";
            RUST_BACKTRACE = 1;
            buildInputs = [
              toolchain
              pkgs.rust-analyzer
              pkgs.cargo-audit
              pkgs.cargo-license
              pkgs.cargo-edit
            ];
          };
        };
    };
}
