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
    [[ ${PWD##*/} == "$demo" ]] && (( wasmBuild )) && mv index.html ../
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
    (( useMold )) && type mold && export RUSTFLAGS="-C link-arg=-fuse-ld=mold ${RUSTFLAGS-}"

    echo "Compiling native build with rustc flags: ${RUSTFLAGS-}.."

    (( liveReload )) && dx serve --hotpatch "${flags[@]}" && return
    cargo run "${flags[@]}"
}

serve_wasm() {
    echo "Compiling $wasmBuild WebAssembly build with rustc flags: ${RUSTFLAGS-}.."

    (( liveReload )) && dx serve --hotpatch --platform web -- --no-default-features && return
    
    type trunk || cargo install --locked trunk
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
(( verbose )) && set -x && flags+=(--verbose)
(( debug )) || flags+=(--release)

[[ $demo && -d $demo ]] && {
    cd "$demo"

    (( wasmBuild )) && mv ../index.html ./
}

[[ $memPath ]] && link_in_memory "$memPath"

(( liveReload )) && { type dx || cargo install --git https://github.com/DioxusLabs/dioxus dioxus-cli --locked; }

setup_toolchain

echo "Running with flags: ${flags[*]}"
if (( wasmBuild )); then
    serve_wasm
else
    run_native
fi
