{
  description = "Atomic Bomberman clone in Bevy";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
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
            vulkan-loader
            libxkbcommon
            wayland

            # Fast compilation
            clang
            llvmPackages.lld
          ];

          shellHook =
            let
              libs = with pkgs; [ alsaLib udev vulkan-loader libxkbcommon wayland ];
            in
            ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath libs}"
            '';
        };
      });
}
