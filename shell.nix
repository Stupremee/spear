{ pkgs ? import <nixpkgs> { } }:
let
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustfmt = pkgs.rust-bin.stable.latest.rustfmt;
in pkgs.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs; [
    rust
    rustfmt

    python3
    dtc
    cargo-expand
    cargo-watch
  ];
}
