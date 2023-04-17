# Rustiplix kernel

![Test Workflow](https://github.com/zyedidia/rustiplix/actions/workflows/test.yaml/badge.svg)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/zyedidia/rustiplix/blob/master/LICENSE)

Rustiplix is a small operating system kernel written in Rust targeting RISC-V.
It currently supports the VisionFive 2 and the QEMU RISC-V virt machine. It
has a similar architecture as [Multiplix](https://github.com/zyedidia/multiplix)
and is serving as an experiment in using Rust for kernel development.

# Building

Run `knit kernel.boot.bin` to build the kernel. Run `knit qemu` to simulate it.
