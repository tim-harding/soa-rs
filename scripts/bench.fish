#!/usr/bin/fish

cargo +nightly bench \
    --target x86_64-unknown-linux-gnu \
    --profile bench \
    --package soapy-testing \
    --bench benchmark \
    dots-soa
