{ lib, stdenv, fetchFromGitHub, dtc, nixosTests, fetchpatch }:

stdenv.mkDerivation rec {
  pname = "spike";
  version = "master";

  src = fetchFromGitHub {
    owner = "riscv";
    repo = "riscv-isa-sim";
    rev = "6c18ef569c210daf9713b4f26bc0c4f2c3769457";
    sha256 = "sha256-auVAWzSAI5FunnbEVbhfP3wbiP0v/HX93MKt+/9QMMo=";
  };

  nativeBuildInputs = [ dtc ];
  enableParallelBuilding = true;

  postPatch = ''
    patchShebangs scripts/*.sh
    patchShebangs tests/ebreak.py
  '';

  doCheck = true;

  passthru.tests = { can-run-hello-world = nixosTests.spike; };
}
