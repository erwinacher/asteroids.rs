# Asteroids (Bevy + Rust + WebAssembly)

A classic **Asteroids** game written in **Rust** using **Bevy**, targeting **native** and **WebAssembly** with on-screen mobile controls.

Runs in the browser. No JS framework. No engine lock-in. Just Rust.

## Features

- Bevy 2D game
- WebAssembly build (browser-ready)
- Touch / mobile controls via `wasm-bindgen`
- Keyboard support (desktop)
- Auto-resizing canvas (fits parent)
- Single codebase for native + web

## Build (Native)

```bash
cargo run --release
```

## Build (Web / WASM)

### 1. Install target + tools

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

### 2. Build

```
cargo build --release --target wasm32-unknown-unknown

wasm-bindgen \
  --target web \
  --no-typescript \
  --out-dir web \
  target/wasm32-unknown-unknown/release/asteroids.wasm
```

### 3. Serve

```
cd web
python3 -m http.server
# or
npx serve .
```

## How Startup Works (Important)

### This project uses:

```
#[wasm_bindgen(start)]
```

That means:

- Rust owns startup
- JavaScript must not call run()
- JS only needs await init()

If you import or call run from JavaScript, the WASM module will fail to initialize.

<img width="981" height="772" alt="image" src="https://github.com/user-attachments/assets/b57c6201-acff-4710-8744-1ee25d9f3c8e" />

