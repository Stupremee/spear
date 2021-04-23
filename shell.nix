{ pkgs ? import <nixpkgs> { } }:
let
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustfmt = pkgs.rust-bin.stable.latest.rustfmt;

  pkgsRiscv = import <nixpkgs> {
    crossSystem = (import <nixpkgs/lib>).systems.examples.riscv32-embedded;
  };

  spike = pkgs.callPackage ./nix/spike.nix { };
in pkgsRiscv.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs; [
    rust
    rustfmt

    llvm_11
    python3
    dtc
    cargo-expand
    cargo-watch
    spike
  ];
}
