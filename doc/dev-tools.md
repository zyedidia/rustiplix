# Targets

```
rustup target add riscv64imac-unknown-none-elf
```

# Rust analyzer

```
rustup component add rust-src
rustup component add rust-analyzer
ln -s $(rustup which --toolchain stable rust-analyzer) ~/.cargo/bin/rust-analyzer
```

# Clippy

```
rustup component add clippy
```

# Rustfmt

```
rustup component add rustfmt
```
