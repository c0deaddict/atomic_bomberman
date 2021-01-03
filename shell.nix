{ nixpkgs ? import <nixpkgs> {
  overlays = [
    (import (builtins.fetchTarball
      "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz"))
  ];
} }:
with nixpkgs;

let

  libraries = with pkgs; [
    libgcc
    stdenv.cc.libc
    stdenv.cc.cc
    alsaLib
    # vulkan-loader
    x11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libudev
  ];

  libraryPaths = lib.concatStringsSep " " (map (pkg: "-L ${pkg}/lib") libraries);

in mkShell {
  buildInputs = libraries ++ (with pkgs; [
    lutris
    pkgconfig
    vulkan-headers    
    vulkan-tools
    llvmPackages.lld
  ]) ++ (with pkgs.latest.rustChannels.nightly; [
    rust
    rust-analyzer
    cargo
    rustfmt
    rust-analyzer
    clippy
  ]);

  # Linker crashes with SIGSEGV failures
  # https://stackoverflow.com/questions/59126946/rust-llvm-linker-rust-lld-segfaults
  shellHook = ''
    # export RUSTFLAGS="-Clinker=rust-lld -lvulkan ${libraryPaths}"
  '';
}
