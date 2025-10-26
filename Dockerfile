FROM nixos/nix:2.32.1

WORKDIR /app

COPY shell.nix ./

RUN nix-shell --run "cargo init ."

RUN touch ./src/lib.rs

COPY Cargo.toml Cargo.lock build.rs ./

RUN nix-shell --run "nix-shell -p cmake --run \"cargo build --release\""

COPY src ./src

RUN nix-shell --run "cargo build --release"

CMD ["nix-shell", "--run", "cargo run --release"]