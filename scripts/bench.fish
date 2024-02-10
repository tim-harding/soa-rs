#!/usr/bin/env fish

cargo bench \
    --profile bench \
    --package soapy-testing \
    --bench benchmark \
    dots-aligned-array
