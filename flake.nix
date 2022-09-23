{
  description = "basic rust template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "utils/flake-utils";
      };
    };
  };

  outputs = {
    self,
    utils,
    ...
  } @ inputs:
    utils.lib.mkFlake {
      inherit self inputs;
      channels.nixpkgs.overlaysBuilder = channels: [inputs.rust-overlay.overlays.rust-overlay];
      channels.nixpkgs-unstable.overlaysBuilder = channels: [inputs.rust-overlay.overlays.rust-overlay];

      outputsBuilder = channels: {
        devShells.default = channels.nixpkgs.mkShell {
          RUST_LOG = "debug";
          RUST_BACKTRACE = 1;
          shellHook = ''
            export PATH=''${HOME}/.cargo/bin''${PATH+:''${PATH}}
          '';
          buildInputs = [
            (channels.nixpkgs.rust-bin.selectLatestNightlyWith
              (toolchain:
                toolchain.default.override {
                  extensions = ["rust-src" "miri"];
                }))
            channels.nixpkgs-unstable.rust-analyzer
            channels.nixpkgs.cargo-audit
            channels.nixpkgs.cargo-license
            channels.nixpkgs.cargo-tarpaulin
            channels.nixpkgs.cargo-kcov
            channels.nixpkgs.valgrind
            channels.nixpkgs.gnuplot
            channels.nixpkgs.kcov
          ];
        };
      };
    };
}
