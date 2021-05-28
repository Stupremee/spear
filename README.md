# Spear

Spear is a RISC-V emulator that tries to be correct as possible, without focusing on
high-perfomance, similair to the official [spike] emulator, but with more features
and more devices to support.

# Table of Contents

- [Table of Contents](#table-of-contents)
    * [Status](#status)
    * [Usage](#usage)
    * [Plugins](#plugins)
    * [Compliance](#compliance)

## Status

Note that spear is in very early development phase and it will take a long time until you
can run real programs on it. Currently, most of the things mentioned in this README are plans
that I would like to implement, however they are far from being implemented.

Right now spear only supports the following extensions and features:

### Extensions

Thanks to the dynamic and pluggable design of spear, it's very easy to enable/disable
extensions which allows to support many extesions, even ones that are not rattified yet, without
much hassle.

Support for all rattified extensions is planned, and important other extensions, like the V
extension will also come to play with cool new features.

- [ ] Base
  - [x] RV32I v2.1
  - [ ] RV64I v2.1
  - [ ] RV32E (low priority)
  - [ ] RV128I (low priority)
- [ ] Extensions
  - [ ] RV32/64G
    - [ ] M v2.0
    - [ ] A v2.1
    - [ ] F v2.2
    - [ ] D v2.2
    - [x] Zicsr v2.0 (note that many CSRs do not work correctly or might not even be implemented)
    - [x] Zifencei v2.0 (ICACHE emulation is not yet implemented, so `fence.i` is basically a NOP)
  - [ ] Q v2.2
  - [ ] C v2.0
  - [ ] All other extensions (low priority)

### Privilege modes

Spear supports all ratified privilege modes (M, S, and U), however, there are probably *many* bugs
left to fix.

## Usage

Spear currently requires at least the latest beta version (1.53.0-beta.3) of the Rust compiler.

Install the latest spear version from `main` branch by running the following command.
```sh
cargo install --locked --git https://github.com/Stupremee/spear
```

Now you can run `spike`! :tada:
You can take a look at the [example](https://github.com/Stupremee/spear/tree/main/example).
It contains the `hello.S` assembly file and the `link.lds` which is used to compile the `hello.S`
file. To compile the and run `hello.S` file, run the following commands (on NixOS the prefix is `riscv32-none-elf`,
but it might differ on your platform):

```sh
riscv32-none-elf-gcc -Wl,-Texample/link.lds -nostdlib -ffreestanding -mabi=ilp32 -march=rv32i example/hello.S -o example/hello

# And the run the resulting binary using spear
spear example/hello
```

You should see the state of all registers at the end of the program. As you can see `t1` is `0xFF`,
which is correct since we executed `li t1, 255` in the example binary.

If you want to see what the emulator is doing under the hood, set the `SPEAR_LOG` environment
variable to `debug` or even `trace`. The `trace` log level will show **every** instruction
that is executed, thus the output can be huge for larger programs.

## Plugins

Spear allows one to write plugins and inject them into the emulator, allowing custom devices to be
written in any language that can be compiled to [WASM]. Infact, every standard device that is
supported in spear (e.g. the UART driver, RTIC, etc) are written using WASM plugins.

## Compliance

Spear has support for running the [riscv-tests] (already implemented and used for testing)
or [riscof] (not yet implemented) test suites to prove the correctnes of the emulator and it's execution.


[spike]: https://github.com/riscv/riscv-isa-sim
[WASM]: https://webassembly.org/
[riscv-tests]: https://github.com/riscv/riscv-tests
[riscof]: https://gitlab.com/incoresemi/riscof/
