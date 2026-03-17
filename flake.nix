{
  description = "rdn - Rusty Dos Navigator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        rdn = pkgs.rustPlatform.buildRustPackage {
          pname = "rdn";
          version = "0.1.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs;
            lib.optionals stdenv.isLinux [
              wayland
              libxkbcommon
              libx11
              libxi
              libxrandr
            ]
            ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.AppKit
              darwin.apple_sdk.frameworks.ApplicationServices
            ];

          meta = with pkgs.lib; {
            description = "A modern two-panel file manager inspired by Dos Navigator";
            homepage = "https://github.com/apatrushev/rdn";
            license = licenses.mit;
            mainProgram = "rdn";
          };
        };
      in {
        packages = {
          default = rdn;
          rdn = rdn;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = rdn;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            pkg-config
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      });
}
