let
  sources = import ./nix/sources.nix;
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.rust-overlay) ]; };

in pkgs.mkShell {
  buildInputs = with pkgs; [
    (rust-bin.nightly.latest.rust.override {
      extensions = [ "rust-src" ];
    })

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
    export VK_ICD_FILENAMES="/run/opengl-driver/share/vulkan/icd.d/intel_icd.x86_64.json"
  '';
}
