{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "python-rust-dev";

  buildInputs = [
    # Python 3.12 with common packages
    (pkgs.python312.withPackages (ps: [
      ps.pip
      ps.virtualenv
      ps.setuptools
      ps.matplotlib
      ps.numpy
      ps.scipy
      ps.requests
      ps.pillow
      ps.imageio
      ps.genanki
    ]))
    pkgs.pyenv
    pkgs.virtualenv
    pkgs.sqlite
    pkgs.jupyter-all
    pkgs.imagemagick
    pkgs.git-lfs
    pkgs.maturin
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
    source venv/bin/activate
    alias runTests="python -m unittest discover -s test -v"
    alias cargo='nix run --impure github:nix-community/nixGL -- cargo'
  '';
}
