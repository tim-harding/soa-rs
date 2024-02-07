#!/usr/bin/fish

set --export RUSTFLAGS "--emit asm"
set exec $(
    cargo +nightly build \
    --benches \
    --profile bench \
    --target x86_64-unknown-linux-gnu \
    --package soapy-testing \
    --message-format json
)

echo $exec
