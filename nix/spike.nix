{ lib, stdenv, fetchFromGitHub, dtc, nixosTests, fetchpatch }:

stdenv.mkDerivation rec {
  pname = "spike";
  version = "master";

  src = fetchFromGitHub {
    owner = "riscv";
    repo = "riscv-isa-sim";
    rev = "0981d396bca516a2b17db4cf744b8463b210c4cc";
    sha256 = "sha256-sNFW0X1Tl0atu/Ors6nqowkElMLp6ExkOrI0lwtIbWE=";
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
