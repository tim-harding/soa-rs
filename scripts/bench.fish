#!/usr/bin/fish

cargo bench \
    --profile bench \
    --package soapy-testing \
    --bench benchmark \
    dots-soa
