#!/usr/bin/env bash
# Netlify build: bootstrap the Rust toolchain, then build the wasm pkg and
# the Vite site. Run from web/ (the Netlify base directory).
set -euxo pipefail

WASM_PACK_VERSION=0.13.1

if ! [ -x "$CARGO_HOME/bin/cargo" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
    sh -s -- -y --profile minimal --target wasm32-unknown-unknown
fi
. "$CARGO_HOME/env"
rustup target add wasm32-unknown-unknown

if ! [ -x "$CARGO_HOME/bin/wasm-pack" ]; then
  curl -sSfL "https://github.com/rustwasm/wasm-pack/releases/download/v${WASM_PACK_VERSION}/wasm-pack-v${WASM_PACK_VERSION}-x86_64-unknown-linux-musl.tar.gz" |
    tar -xz --strip-components=1 -C "$CARGO_HOME/bin" --wildcards '*/wasm-pack'
fi

npm run build:wasm
npm run build
