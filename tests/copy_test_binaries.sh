#!/usr/bin/env sh

function copy_suite() {
  files=$(fd --no-ignore "$1-[a-z_]+$" riscv-tests/isa)

  mkdir -p binaries/$1
  for file in $files; do
    cp $file binaries/$1
  done
}

copy_suite rv32mi-p
copy_suite rv32si-p
copy_suite rv32ui-p
