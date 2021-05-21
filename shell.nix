{ pkgs ? import <nixpkgs> { } }:
let
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustfmt = pkgs.rust-bin.stable.latest.rustfmt;

  pkgsRiscv = import <nixpkgs-unstable> {
    #crossSystem = (import <nixpkgs/lib>).systems.examples.riscv32-embedded;
    localSystem = "${pkgs.system}";
    crossSystem = {
      config = "riscv32-none-elf";
      abi = "ilp32";
    };
  };

  spike = pkgs.callPackage ./nix/spike.nix { };
in
pkgs.mkShell {
  name = "rust-shell";
  nativeBuildInputs = with pkgs; [
    rust
    rustfmt

    binutils
    python3
    dtc
    cargo-expand
    cargo-watch
    spike
    autoconf
    hexyl

    pkgsRiscv.buildPackages.gcc
  ];
}
