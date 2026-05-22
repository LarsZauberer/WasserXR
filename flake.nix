{
  description = "A flake for WasserXR";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        nightlyRust = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "rustfmt"
              "miri"
              "rust-analyzer"
            ];
          }
        );
      in
      {
        packages = {
        };
        devShells.default = pkgs.mkShell {
          name = "devShell";

          buildInputs = [
            # nightlyRust
            pkgs.rustc
            pkgs.cargo
            pkgs.rustfmt
          ];
        };
      }
    );
}
