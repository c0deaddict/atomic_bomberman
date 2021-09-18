let
  sources = import ./nix/sources.nix;
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.rust-overlay) ]; };

in pkgs.mkShell {
  buildInputs = with pkgs; [
    (rust-bin.nightly.latest.default.override {
      extensions = [
        "cargo"
        "rust-src"
        "rust-analyzer-preview"
        "clippy"
        "rustfmt"
      ];
    })

    cargo-edit

    # bevy-specific deps (from https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md)
    pkgconfig
    udev
    alsaLib
    lutris
    x11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    vulkan-tools
    vulkan-headers
    vulkan-loader
    vulkan-validation-layers

    # Fast compilation
    clang
    llvmPackages.lld
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      pkgs.lib.makeLibraryPath [ pkgs.alsaLib pkgs.udev pkgs.vulkan-loader ]
    }"'';
}
