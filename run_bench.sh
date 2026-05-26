#!/bin/bash

cd tests-unified/

TP_ENV=""
if [ -n "$TARGET_PATH" ]; then
    TP_ENV="TARGET_PATH=$TARGET_PATH"
fi

eval "$TP_ENV cargo test -q --test benchmark --features native -- --nocapture"

echo ""
eval "$TP_ENV cargo test -q --test benchmark --features anchor -- --nocapture"

echo ""
eval "$TP_ENV cargo test -q --test benchmark --features anchor-zero-copy -- --nocapture"
