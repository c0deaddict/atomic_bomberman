let
  sources = import ./nix/sources.nix;
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.rust-overlay) ]; };

in pkgs.mkShell {
  buildInputs = with pkgs; [
    rust-bin.nightly.latest.rust

    alsaLib
    lutris
    pkgconfig
    vulkan-headers
    vulkan-loader
    vulkan-tools
    x11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libudev

    # https://bevyengine.org/learn/book/getting-started/setup/
    clang
    llvmPackages.lld
  ];

  shellHook = ''
    export RUSTFLAGS="-l vulkan"
  '';
}
