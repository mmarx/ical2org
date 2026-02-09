{
  description = "ical2org - convert ical calenders into org agendas";

  inputs = {
    dried-nix-flakes.url = "github:cyberus-technology/dried-nix-flakes";
    nixpkgs.url = "https://channels.nixos.org/nixos-25.11/nixexprs.tar.xz";
  };

  outputs =
    inputs:
    inputs.dried-nix-flakes inputs (
      inputs:
      let
        cargoMeta = (fromTOML (builtins.readFile ./Cargo.toml)).package;
        pkgs = inputs.nixpkgs.legacyPackages;
        lib = pkgs.lib;
      in
      {
        packages =
          let
            ical2org = pkgs.rustPlatform.buildRustPackage {
              pname = "ical2org";
              inherit (cargoMeta) version;

              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              meta = {
                inherit (cargoMeta) description homepage;
                license = [ lib.licenses.gpl3Plus ];
              };
            };
          in
          {
            inherit ical2org;
            default = ical2org;
          };

        overlays =
          let
            ical2org = final: prev: inputs.self.packages.ical2org;
          in
          {
            inherit ical2org;
            default = ical2org;
          };

        devShells.default = pkgs.mkShell {
          RUST_LOG = "debug";
          RUST_BACKTRACE = 1;
          buildInputs = lib.attrValues {
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
      }
    );
}
