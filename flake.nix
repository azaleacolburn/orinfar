{
  description = "A Text Editor For Witches";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
      rust-overlay,
    }:
    let
      inherit (nixpkgs) lib;
      forEachPkgs =
        f:
        lib.genAttrs (import systems) (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            }
          )
        );
    in
    {
      devShells = forEachPkgs (pkgs: {
        default = pkgs.mkShell {
          buildInputs = [
            pkgs.rust-bin.nightly.latest.default
          ];
        };
      });

      packages = forEachPkgs (pkgs: {
        default =
          let
            p = (lib.importTOML ./Cargo.toml).package;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.nightly.latest.default;
              rustc = pkgs.rust-bin.nightly.latest.default;
            };
          in
          rustPlatform.buildRustPackage {
            pname = p.name;
            inherit (p) version;

            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              mainProgram = "orinfar";
              license = lib.licenses.mit;
            };
          };
      });
    };
}
