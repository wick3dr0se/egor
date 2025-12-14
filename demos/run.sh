#!/bin/bash
set -euo pipefail

flags=()
verbose=0
debug=1
memPath=
buildPath=
wasmBuild=0
liveReload=0
useMold=1
demo=
indexPath="$PWD/index.html"

usage() {
    cat <<EOF
Usage: $0 [OPTION]... [ARG]...

Options:
  -h, --help                Show this usage.
  -v, --verbose             Enable verbose output.
  -r, --release             Build in release mode (default: debug).
  -F, --features <FEAT>...  Comma-separated list of features to enable (e.g., -f log,webgl).
  -w, --wasm                Build for WebAssembly
  -m, --mem [DIR]           Use tmpfs (RAM-backed) dir to speed up (incremental) builds by avoiding disk I/O (default: /tmp).
  -l, --live-reload         Use dioxus-cli for live-reload during development (native or wasm).
  --no-mold                 Disable mold linker for native builds & fallback to default rustc linker (default: enabled if available)

Arguments:
  <TARGET>                  Specify a demo to run (e.g., shooter).

Environment Variables:
  RUSTFLAGS=...             Pass custom rustc flags (e.g., RUSTFLAGS='-C target-cpu=native').
EOF
    exit 1
}

panic() { printf '[\e[31mPANIC\e[m](L%s): %s\n' "${BASH_LINENO[0]}" "$1" >&2; usage; }

cleanup() {
    [[ -d $buildPath ]] && rm -fr "$buildPath"
    (( wasmBuild )) && [[ -f index.html ]] && mv index.html "$indexPath"
}

is_tmpfs() { [[ -d "$1" && -w "$1" ]] && mountpoint -q "$1"; }

link_in_memory() {
    is_tmpfs "$1" || panic "$1 is NOT a writable & mounted tmpfs"

    buildPath="$1/${0##*/}.$$"
    echo "Linking source in memory at $buildPath.."

    mkdir "$buildPath"
    for f in * .*; do
        [[ "$f" =~ ^($0|dist|target)$|^.git* ]] || ln -s "$PWD/$f" "$buildPath/"
    done

    export CARGO_TARGET_DIR="$PWD/target"
    cd "$buildPath"
}

setup_toolchain() {
    echo "Configuring toolchain.."

    PATH="$HOME/.cargo/bin:$PATH"
    
    (( wasmBuild )) && rustup target add wasm32-unknown-unknown; :
}

run_native() {
    (( useMold )) && type mold 2>/dev/null && export RUSTFLAGS="-C link-arg=-fuse-ld=mold ${RUSTFLAGS-}"

    echo "Compiling native build with rustc flags: ${RUSTFLAGS-}.."

    (( liveReload )) && dx serve --hotpatch "${flags[@]}" && return
    cargo run "${flags[@]}"
}

serve_wasm() {
    echo "Compiling $wasmBuild WebAssembly build with rustc flags: ${RUSTFLAGS-}.."

    trunk serve "${flags[@]}"
}

trap cleanup EXIT

while (( $# )); do
    case $1 in
        -h|--help) usage;;
        -v|--verbose) verbose=1;;
        -r|--release) debug=0;;
        -F|--features) [[ ${2-} != -* ]] && shift && flags+=(--features "$1");;
        -w|--wasm) wasmBuild=1;;
        -m|--mem) [[ ${2-} && $2 != -* ]] && memPath="$2" && shift; memPath="${memPath:-/tmp}";;
        -l|--live-reload) liveReload=1;;
        --no-mold) useMold=0;;
        -*) panic "Unknown flag: $1";;
        *) [[ $demo ]] && panic 'Multiple demo targets given'; demo="$1";;
    esac
    shift
done

if (( wasmBuild && liveReload )); then
    panic "--live-reload is only supported for native builds (for now)"
fi

(( verbose )) && set -x && flags+=(--verbose)
(( debug )) || flags+=(--release)

[[ $demo ]] && {
    if [[ $demo == */* ]]; then
        # demo specified as crate/demo
        crate="${demo%%/*}" demoName="${demo##*/}"
        demoPath="../crates/$crate/demos/$demoName"
        [[ -d "$demoPath" ]] || panic "Demo '$demoName' not found in crate '$crate'"
    else
        # demo in root demos/
        demoPath="$demo"
        [[ -d "$demoPath" ]] || panic "Demo '$demo' not found in root demos/ (run a crate demo ex: ./run.sh egor_render/triangle)"
    fi

    (( wasmBuild )) && [[ -f $indexPath ]] && mv "$indexPath" "$demoPath"
    cd "$demoPath"
}

[[ $memPath ]] && link_in_memory "$memPath"

setup_toolchain

echo "Running with flags: ${flags[*]}"
if (( wasmBuild )); then
    serve_wasm
else
    run_native
fi
