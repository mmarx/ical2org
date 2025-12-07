{
  description = "ical2org - convert ical calenders into org agendas";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
    # rust-overlay = {
    #   url = "github:oxalica/rust-overlay";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
  };

  outputs =
    { self, utils, ... }@inputs:
    utils.lib.mkFlake {
      inherit self inputs;
      outputsBuilder =
        channels:
        let
          pkgs = channels.nixpkgs;
          cargoMeta = (fromTOML (builtins.readFile ./Cargo.toml)).package;
        in
        {
          packages = rec {
            ical2org = pkgs.rustPlatform.buildRustPackage {
              pname = "ical2org";
              inherit (cargoMeta) version;

              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

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
            buildInputs = pkgs.lib.attrValues {
              inherit (pkgs)
                cargo
                cargo-audit
                cargo-edit
                cargo-license
                clippy
                rustfmt
                rustc
                rust-analyzer
                ;
            };
          };
        };
    };
}
