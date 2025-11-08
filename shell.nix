{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc          # Rust compiler
    pkgs.cargo          # Rust package manager
    pkgs.pkg-config     # pkg-config for discovering native libs
    pkgs.fontconfig     # fontconfig runtime library
    pkgs.freetype       # freetype often required alongside fontconfig
    pkgs.gcc            # compiler for C dependencies
    pkgs.makeWrapper    # sometimes needed for wrapping build tools
    pkgs.libspatialite  # used to match swissboundaries (this lib is included in the build)
    pkgs.cmake
    
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

    # Virtual display server
    pkgs.xorg.xvfb
    pkgs.xorg.xrandr
  ];
  LD_LIBRARY_PATH="${pkgs.mesa}/lib:${pkgs.libGL}/lib:${pkgs.libspatialite}/lib:$LD_LIBRARY_PATH";

}
