#!/usr/bin/env sh

files="$(fd --no-ignore 'rv32ui-p-[a-z_]+$' tests/riscv-tests/isa)"

for file in $files; do
  cp $file tests/binaries
done
