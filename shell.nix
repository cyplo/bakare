let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  channel = (nixpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; });
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "bakare_shell";
    buildInputs = [
      channel.rust
      linuxPackages.perf flamegraph cargo-flamegraph geeqie
      cargo-edit
      cacert openssl openssh zlib
      pkgconfig clang llvm
      git
    ];
    shellHook = ''
      export RUST_SRC_PATH="${channel.rust-src}/lib/rustlib/src/rust/src"
    '';
  }
