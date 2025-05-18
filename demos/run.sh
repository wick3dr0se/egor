#!/bin/bash
set -euo pipefail

flags=()
verbose=0
useMold=1
debug=1
binTarget=
memPath=
buildPath=
wasmBuild=0

usage() {
    cat <<EOF
Usage: $0 [OPTION]... [ARG]...

Options:
  -h, --help                Show this usage.
  -v, --verbose             Enable verbose output.
  -b, --bin <TARGET>        Specify a Rust binary target to run (e.g., --bin my_binary).
  -F, --features <FEAT>...  Comma-separated list of features to enable (e.g., -f log,webgl).
  -w, --wasm                Build for WebAssembly
  -m, --mem [DIR]           Use tmpfs (RAM-backed) dir to speed up (incremental) builds by avoiding disk I/O (default: /tmp).
  --no-mold                 Disable mold linker for native builds & fallback to default rustc linker (default: enabled if available)

Arguments:
  r, release                Build in release mode (default: debug).

Environment Variables:
  RUSTFLAGS=...             Pass custom rustc flags (e.g., RUSTFLAGS='-C target-cpu=native').
EOF
    exit 1
}

panic() { printf '[\e[31mPANIC\e[m](L%s): %s\n' "${BASH_LINENO[0]}" "$1" >&2; usage; }

cleanup() {
    [[ -d $buildPath ]] && rm -fr "$buildPath"
    [[ ${PWD##*/} == "$binTarget" ]] && (( wasmBuild )) && mv index.html ../
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

run_native_build() {
    (( useMold )) && type mold && export RUSTFLAGS="-C link-arg=-fuse-ld=mold ${RUSTFLAGS-}"

    echo "Compiling native build with rustc flags: ${RUSTFLAGS-}.."
    cargo run "${flags[@]}"
}

setup_wasm_toolchain() {
    echo "Configuring WebAssembly toolchain.."

    PATH="$HOME/.cargo/bin:$PATH"
    type trunk || cargo install --locked trunk

    rustup target add wasm32-unknown-unknown
}

run_wasm_build() {
    setup_wasm_toolchain
    
    echo "Compiling $wasmBuild WebAssembly build with rustc flags: ${RUSTFLAGS-}.."
    trunk serve "${flags[@]}"
}

trap cleanup EXIT

while (( $# )); do
    case $1 in
        -h|--help) usage;;
        -v|--verbose) verbose=1;;
        r|release) debug=0;;
        -b|--bin)
            [[ ${2-} && $2 != -* ]] || panic "You must specify a target to --bin"
            binTarget="$2"; shift
        ;;
        -F|--features) [[ ${2-} != -* ]] && shift && flags+=(--features "$1");;
        -w|--wasm) wasmBuild=1;;
        -m|--mem) [[ ${2-} && $2 != -* ]] && memPath="$2" && shift; memPath="${memPath:-/tmp}";;
        --no-mold) useMold=0;;
        *) panic "Unknown argument: $1";;
    esac
    shift
done
(( verbose )) && set -x && flags+=(--verbose)
(( debug )) || flags+=(--release)

[[ $binTarget && -d $binTarget ]] && {
    cd "$binTarget"

    (( wasmBuild )) && mv ../index.html ./
}
[[ $wasmBuild == webgl ]] && flags+=(--features webgl)

[[ $memPath ]] && link_in_memory "$memPath"

echo "Running with flags: ${flags[*]}"
if (( wasmBuild )); then
    run_wasm_build
else
    run_native_build
fi
