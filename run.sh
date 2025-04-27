#!/bin/bash

set -eEuo pipefail

MODE="${1:-native}"
PROFILE="${2:-debug}"
FLAGS=()

if [[ $PROFILE == release ]]; then
    FLAGS+=("--$PROFILE")
fi


[[ $MODE != native ]]&& {
    export PATH="$HOME/.cargo/bin:$PATH"
    hash trunk 2>/dev/null || cargo install --locked trunk

    rustup target add wasm32-unknown-unknown
}


case $MODE in
    native)
        echo "Running native $PROFILE build..."
        cargo run "${FLAGS[@]}" --example cross_platform
    ;;
    webgpu)
        echo "Running WebGPU (native WASM) $PROFILE build..."
      
        trunk serve "${FLAGS[@]}"
    ;;
    webgl)
        echo "Running WebGL $PROFILE build..."
        trunk serve "${FLAGS[@]}" --features webgl --example cross_platform
    ;;
    *)
        echo "Unknown mode: $MODE"
        echo "Usage: $0 [native|webgpu|webgl] [debug|release]"
        exit 1
    ;;
esac
