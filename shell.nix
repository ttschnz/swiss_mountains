{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "python-rust-dev";

  buildInputs = [
    pkgs.sqlite
    pkgs.imagemagick
    pkgs.git-lfs
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.pkg-config
    pkgs.fontconfig

    # Needed for three-d / winit / glutin headless context
    pkgs.mesa
    pkgs.libGL
    pkgs.libGLU
    pkgs.libglvnd
    pkgs.virtualgl

    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libxcb
  ];
  LD_LIBRARY_PATH="/run/opengl-driver/lib:/run/opengl-driver-32/lib";
  shellHook = ''
    alias cargo='nix run --impure github:nix-community/nixGL -- cargo'
  '';
}
