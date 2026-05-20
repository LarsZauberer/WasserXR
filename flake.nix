{
  description = "A flake for WasserXR";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        lib = nixpkgs.lib;
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "WasserXR";
          version = "pre 0.1.0";

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              let
                baseName = builtins.baseNameOf path;
              in
              !(baseName == "build");
          };

          nativeBuildInputs = [
            # Build packages
            pkgs.clang-tools
            pkgs.clang
            pkgs.cmake
            pkgs.doxygen

            # Libraries
            pkgs.glib
            pkgs.pkg-config
            pkgs.pcre2
            pkgs.libsysprof-capture
          ];

          cmakeFlags = [
            (lib.cmakeBool "WXR_TESTS" false)
            (lib.cmakeBool "BUILD_DEBUG" false)
          ];

          meta = {
            license = {
              fullName = "WasserXR License";
              url = "https://github.com/LarsZauberer/WasserXR/blob/main/LICENSE.md";
              free = false;
            };
          };
        };
        devShells.default = pkgs.mkShell {
          name = "devShell";

          buildInputs = [
            pkgs.clang-tools
            pkgs.clang
            pkgs.cmake
            pkgs.gdb
            pkgs.valgrind

            pkgs.doxygen

            pkgs.glib
            pkgs.pkg-config
            pkgs.pcre2
            pkgs.libsysprof-capture
          ];

          shellHook = ''
            export ASAN_SYMBOLIZER_PATH="${pkgs.llvm}/bin/llvm-symbolizer"

            export ASAN_OPTIONS="symbolize=1:check_initialization_order=1:detect_stack_use_after_return=1:strict_string_checks=1:detect_leaks=1"
            export UBSAN_OPTIONS="print_stacktrace=1:halt_on_error=0"
          '';
        };
      }
    );
}
