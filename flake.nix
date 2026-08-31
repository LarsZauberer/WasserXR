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
              "clippy"
              "rust-src"
              "rustfmt"
              "miri"
              "rust-analyzer"
              "llvm-tools-preview"
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
            nightlyRust
            pkgs.cargo-nextest
            pkgs.cargo-llvm-cov
            pkgs.cargo-expand
            pkgs.cargo-fuzz
            pkgs.cargo-tarpaulin
            pkgs.rust-cbindgen
            pkgs.cargo-release
            pkgs.gcc

            pkgs.plantuml
            pkgs.graphviz
          ];
        };
      }
    );
}
