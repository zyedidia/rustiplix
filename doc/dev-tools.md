# Rust analyzer

```
rustup component add rust-src
rustup component add rust-analyzer
ln -s $(rustup which --toolchain stable rust-analyzer) ~/.cargo/bin/rust-analyzer
```

To use `rust-analyzer` you must also have a `rust-project.json` file specifying
the project layout. You can create the file with

```
knit rust-project.json
```

# Clippy

```
rustup component add clippy
```

# Rustfmt

```
rustup component add rustfmt
```
