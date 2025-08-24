# SPDX-FileCopyrightText: 2014-2024 Christina Sørensen, eza contributors
# SPDX-License-Identifier: MIT
{
  description = "A Text Editor For Witches";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";

    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs = {
        systems.follows = "systems";
      };
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      flake-utils,
      naersk,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = (import nixpkgs) {
          inherit system overlays;
        };

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
          clippy = toolchain;
        };

        darwinBuildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
        ];

        buildInputs = [ pkgs.zlib ] ++ darwinBuildInputs;
      in
      {
        # For `nix fmt`
        packages = {
          check = naersk'.buildPackage {
            inherit buildInputs;
            src = ./.;
            mode = "check";
          };

          test = naersk'.buildPackage {
            inherit buildInputs;
            src = ./.;
            mode = "test";
          };

          clippy = naersk'.buildPackage {
            inherit buildInputs;
            src = ./.;
            mode = "clippy";
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            [
              cargo
              clippy
              rustup
              toolchain
            ]
            ++ darwinBuildInputs;
        };
      }
    );
}
