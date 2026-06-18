# Vauxl Client

A Discord-style Matrix client. Tauri v2 shell, React (web) UI, and a Rust core.
Today it runs entirely on an in-memory `MockBackend`, so the whole interface can
be built and felt before any Matrix code exists. Swapping in a matrix-rust-sdk
backend later changes only one constructor.

## Layout

```
client/
  core/        vauxl-core: the contract types, the ChatBackend trait, MockBackend (no Tauri, no Matrix)
  src-tauri/   the Tauri shell: command/event wrappers over ChatBackend, tauri-specta bindings export
  src/         the React UI (web). src/bindings.ts is GENERATED from Rust, do not edit it by hand
  docs/        DATA_INTERFACE.md explains the contract and the security boundary
```

The UI depends only on `core` (through the generated bindings), not on Tauri or
Matrix, so the shell and the protocol stay swappable. See `docs/DATA_INTERFACE.md`.

## Prerequisites

- Rust (stable) and `cargo`
- Bun
- Linux desktop libraries for the Tauri webview. On Fedora:
  `sudo dnf install webkit2gtk4.1-devel libsoup3-devel`

## Run

```sh
bun install
bun run tauri dev      # launches the window; runs Vite + the Rust app, needs a display
```

In debug builds, `src/bindings.ts` is regenerated automatically on launch. To
regenerate it without launching the GUI:

```sh
cargo test -p vauxl-app export_bindings
```

## Build

```sh
cargo build                 # compiles the workspace (core + app)
cargo test                  # runs core + binding-export tests
bun run build               # typechecks and builds the frontend into dist/
bun run tauri build         # produces a packaged desktop bundle
```

## Swapping the mock for real Matrix

1. Add a `MatrixBackend` to `core` (a new module) that implements `ChatBackend`
   using `matrix-rust-sdk`. Crypto stays in `vodozemac` via the SDK; you write no
   crypto yourself.
2. In `src-tauri/src/lib.rs`, change the constructor in `run` from
   `MockBackend::new()` to your `MatrixBackend`. Nothing in the UI changes.

## Leftover template files (safe to delete)

This repo was scaffolded from an Electrobun template. These files are now unused
and can be removed: `electrobun.config.ts`, `src/bun/`, `src/mainview/`,
`llms.txt`, and the old `build/` directory. They are excluded from the build but
remain on disk so nothing is lost without your say-so.
