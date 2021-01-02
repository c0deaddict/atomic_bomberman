{ nixpkgs ? import <nixpkgs> {} }:

with nixpkgs;

mkShell {
  buildInputs = with pkgs; [
    alsaLib
    lutris
    pkgconfig
    vulkan-headers
    vulkan-loader
    vulkan-tools
    # vulkan-validation-layers
    x11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    libudev

    # https://bevyengine.org/learn/book/getting-started/setup/
    # TODO: not yet using rust nightly
    llvmPackages.lld
  ];
}
