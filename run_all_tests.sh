#!/bin/bash

cd tests-unified/

TP_ENV=""
if [ -n "$TARGET_PATH" ]; then
    TP_ENV="TARGET_PATH=$TARGET_PATH"
fi

echo "--- RUNNING NATIVE TESTS ---"
if ! eval "$TP_ENV cargo test --features native"; then
    echo "ERROR: Native tests failed!"
    exit 1
fi

echo ""
echo "--- RUNNING ANCHOR TESTS ---"
if ! eval "$TP_ENV cargo test --features anchor"; then
    echo "ERROR: Anchor tests failed!"
    exit 1
fi

echo ""
echo "--- RUNNING ANCHOR ZERO COPY TESTS ---"
if ! eval "$TP_ENV cargo test --features anchor-zero-copy"; then
    echo "ERROR: Anchor Zero Copy tests failed!"
    exit 1
fi

echo ""
echo "ALL TESTS PASSED SUCCESSFULLY"
