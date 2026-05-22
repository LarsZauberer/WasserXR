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
        packages = {
          default = pkgs.stdenv.mkDerivation {
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
              (lib.cmakeBool "BUILD_DEBUG" true)
              (lib.cmakeBool "WXR_TESTS" false)
            ];

            meta = {
              license = {
                fullName = "WasserXR License";
                url = "https://github.com/LarsZauberer/WasserXR/blob/main/LICENSE.md";
                free = false;
              };
            };
          };
          docs = self.packages.${system}.default.overrideAttrs (_: {
            pname = "WasserXR-docs";
            postInstall = ''
              mkdir -p $out/share/doc/wasserxr
              cp -r docs/html $out/share/doc/wasserxr/html
            '';
          });
          docker =
            let
              docsPackage = self.packages.${system}.docs;
              nginxConf = pkgs.writeTextDir "etc/nginx/nginx.conf" ''
                user nobody nobody;
                events {}
                http {
                  include ${pkgs.nginx}/conf/mime.types;
                  server {
                    listen 80;
                    root ${docsPackage}/share/doc/wasserxr/html;
                    index index.html;
                  }
                }
              '';
            in
            pkgs.dockerTools.buildLayeredImage {
              name = "wasserxr-docs";
              tag = "latest";
              contents = [
                pkgs.nginx
                pkgs.dockerTools.fakeNss
                docsPackage
                nginxConf
              ];
              extraCommands = ''
                mkdir -p var/log/nginx var/cache/nginx tmp
              '';
              config = {
                Cmd = [
                  "${pkgs.nginx}/bin/nginx"
                  "-c"
                  "/etc/nginx/nginx.conf"
                  "-e"
                  "/dev/stderr"
                  "-g"
                  "daemon off;"
                ];
                ExposedPorts = {
                  "80/tcp" = { };
                };
              };
            };
        };
        checks = {
          clang-tidy = self.packages.${system}.default.overrideAttrs (oldAttrs: {
            pname = "WasserXR-clang-tidy";
            nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [ pkgs.python3 ];
            cmakeFlags = [
              (lib.cmakeBool "BUILD_DEBUG" true)
              (lib.cmakeBool "WXR_TESTS" false)
            ];
            doCheck = true;
            checkPhase = ''
              runHook preCheck
              python3 $(command -v run-clang-tidy) -p . -warnings-as-errors='*' 'src/WasserXR/.*\.c$'
              runHook postCheck
            '';
          });
          default = self.packages.${system}.default.overrideAttrs (_: {
            pname = "WasserXR-tests";
            cmakeFlags = [
              (lib.cmakeBool "BUILD_DEBUG" true)
              (lib.cmakeBool "WXR_TESTS" true)
            ];
            doCheck = true;
            checkPhase = ''
              runHook preCheck
              ctest --output-on-failure
              runHook postCheck
            '';
          });
        };
        devShells.default = pkgs.mkShell {
          name = "devShell";

          buildInputs = [
            pkgs.clang-tools
            pkgs.clang
            pkgs.cmake
            pkgs.gdb

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
