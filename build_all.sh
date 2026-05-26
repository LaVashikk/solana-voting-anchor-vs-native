#!/bin/bash

echo "--- BUILDING NATIVE ---"
if ! (cd NATIVE && cargo build-sbf); then
    echo "ERROR: Native build failed!"
    exit 1
fi

echo ""
echo "--- BUILDING ANCHOR ---"
if ! (cd ANCHOR && cargo build-sbf); then
    echo "ERROR: Anchor build failed!"
    exit 1
fi

echo ""
echo "--- BUILDING ANCHOR ZERO COPY ---"
if ! (cd ANCHOR_ZERO_COPY && cargo build-sbf); then
    echo "ERROR: Anchor Zero Copy build failed!"
    exit 1
fi

echo ""
echo "ALL PROJECTS BUILT SUCCESSFULLY"
