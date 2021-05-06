# Spear

Spear is a RISC-V emulator that tries to be correct as possible, without focusing on
high-perfomance, similair to the official [spike] emulator, but with more features
and more devices to support.

Note that spear is in very early development phase and it will take a long time until you
can run real programs on it.

## Extension support

Thanks to the dynamic and pluggable design of spear, it's very easy to enable/disable
extensions which allows to support many extesions, even ones that are not rattified yet, without
much hassle.

Support for all rattified extensions is planned, and important other extensions, like the V
extension will also come to play with cool new features.

## Plugins

Spear allows one to write plugins and inject them into the emulator, allowing custom devices to be
written in any language that can be compiled to [WASM]. Infact, every standard device that is
supported in spear (e.g. the UART driver, RTIC, etc) are written using WASM plugins.

## Compliance

Spear has support for running the [riscv-arch-test] or [riscof] test suites to prove the correctnes
of the emulator and it's execution.


[spike]: https://github.com/riscv/riscv-isa-sim
[WASM]: https://webassembly.org/
[riscv-arch-test]: https://github.com/riscv/riscv-arch-test
[risof]: https://gitlab.com/incoresemi/riscof/
