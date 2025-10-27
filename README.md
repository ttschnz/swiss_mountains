# SwissMountains

A rust project that accesses [swissALTI3D](https://opendata.swiss/de/dataset/swissalti3d), [SWISSIMAGE](https://opendata.swiss/de/dataset/swissimage-10-cm-digitale-orthophotomosaik-der-schweiz) and [swissNAMES3D](https://opendata.swiss/en/dataset/swissnames3d-geografische-namen-der-landesvermessung) from swisstopo to export 3D-terrain images and compiles them to a anki deck to learn.

![Example output: Animation of the mountain Niesen](docs/assets/Niesen.gif?v=1)

## Sample Anki-Package
[![Download Anki Deck](https://img.shields.io/badge/Download%20Deck-.apkg-success?style=for-the-badge&logo=anki&logoColor=white)](https://github.com/ttschnz/swiss_mountains/raw/main/docs/assets/Mountains.apkg)

## Timeline
- [x] Optimize for speed
  - [x] speed up python
  - [x] Oxidize (rewrite in rust)
- [x] Export to anki-flashcards
- [x] Dockerize
- [ ] web-api
- [x] fix table locks
- [ ] Documentation, inline comments

## Building Instructions
### Requirements
on Nixos, make sure to set `hardware.graphics.enable = true;` in your `/etc/nixos/configuration.nix`. Then you can just type
```bash
nix-shell .
```
to install dependencies. For other systems, make sure you have **rust** installed. Other dependencies, suwch as `libspatialite` might be required (not tested).
### Building
First we need to build the executable. We do this with
```
cargo build
```
Once the build is complete, you can run the python script by running
```
cargo run
```
