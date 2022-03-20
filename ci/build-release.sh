#!/bin/sh

target=$1

curl -LSfs https://japaric.github.io/trust/install.sh |
    sh -s -- --force --git rust-embedded/cross --tag v0.2.1 --target $target
command -v cross || PATH=~/.cargo/bin:$PATH

cross build --target $target --release

tar -czf kak-dap-$target.tar.gz target/$target/release/kak-dap
