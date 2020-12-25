{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.alsaLib
    pkgs.lutris
    pkgs.pkgconfig
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-tools
    # pkgs.vulkan-validation-layers
    pkgs.x11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.libudev

    # https://bevyengine.org/learn/book/getting-started/setup/
    # TODO: not yet using rust nightly
    pkgs.llvmPackages.lld
  ];
}
