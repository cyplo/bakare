{
  description = "Bakare: modern and simple, yet efficient backup solution";
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, naersk, flake-compat }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        # `nix build`
        packages.bakare = naersk-lib.buildPackage {
          pname = "bakare";
          root = ./.;
        };
        defaultPackage = packages.bakare;

        # `nix run`
        apps.bakare = utils.lib.mkApp {
          drv = packages.bakare;
        };
        defaultApp = apps.bakare;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cacert
            cargo
            cargo-edit
            cargo-outdated
            cargo-release
            cargo-tarpaulin
            cargo-watch
            clippy
            git
            llvmPackages_13.llvm
            nixpkgs-fmt
            openssh
            openssl
            pkg-config
            rustc
            rustfmt
          ];
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
}
