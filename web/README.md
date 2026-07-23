# monsterbook web

Vite + Svelte front end for the monsterbook Rust crate, compiled to
WebAssembly. The wasm `Session` runs in a web worker
(`src/lib/worker.js`) behind a promise-based RPC client
(`src/lib/session-client.js`).

## Building

The wasm package is not committed; build it first into `src/lib/pkg/`:

```sh
npm run build:wasm   # requires cargo, wasm-pack, and the wasm32 target
npm run build        # vite production build
```

On NixOS (no global rust toolchain), run the wasm build through nix:

```sh
nix shell nixpkgs#cargo nixpkgs#rustc nixpkgs#wasm-pack \
    nixpkgs#wasm-bindgen-cli nixpkgs#gcc nixpkgs#lld \
    --command npm run build:wasm
```

(`lld` is needed as the wasm linker; nixpkgs rustc ships the
wasm32-unknown-unknown target.)

## Development without a wasm build

Set `VITE_MOCK_SESSION=1` to use the mock session
(`src/lib/mock-session.js`) instead of the wasm module:

```sh
VITE_MOCK_SESSION=1 npm run dev
```
