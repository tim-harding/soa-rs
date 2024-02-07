#!/usr/bin/fish

set exec $(
    cargo +nightly bench \
    --no-run \
    --profile bench \
    --target x86_64-unknown-linux-gnu \
    --package soapy-testing \
    --message-format json \
    | ./executable_name.jq
)

perf record --call-graph dwarf $exec --bench --profile-time 3
perf script | rustfilt > profile.perf 
flamegraph --perfdata ./perf.data --palette aqua
