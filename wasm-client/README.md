# WASM WebTransport Client

A WebTransport client written in **Rust**, compiled to **WebAssembly** using the `web-transport` crate.

## Features

- ✅ Written entirely in Rust
- ✅ Compiled to WebAssembly for browser execution
- ✅ Uses `web-transport` crate (unified API for native + WASM)
- ✅ Connects to the wtransport Rust server
- ✅ Bidirectional streams for message exchange

## Building

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack` (for building WASM)

Install the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

Install wasm-pack:
```bash
cargo install wasm-pack
```

### Build the WASM module

```bash
wasm-pack build --target web
```

This will create a `pkg/` directory with the compiled WASM and JavaScript bindings.

## Running

1. Build the WASM module (already built in `pkg/`):
   ```bash
   wasm-pack build --target web
   ```

2. Start the wtransport server from the parent directory:
   ```bash
   cd ..
   cargo run
   ```
   The server will run on `https://localhost:8765`

3. In a new terminal, serve the wasm-client directory:
   ```bash
   python3 -m http.server 9000
   ```
   Or use any other static file server.

4. Open your browser to `http://localhost:9000`

5. Click "Connect" to establish WebTransport connection using Rust WASM!

## Architecture

```
┌─────────────────────────────────────┐
│  Browser (index.html)               │
│  ┌───────────────────────────────┐  │
│  │  JavaScript + WASM Module     │  │
│  │  (wasm_client.js + .wasm)     │  │
│  │                               │  │
│  │  Uses: web-transport crate    │  │
│  │  -> Browser WebTransport API  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
           │
           │ WebTransport over QUIC
           ↓
┌─────────────────────────────────────┐
│  Rust Server (wtransport)           │
│  Port: 8765                         │
└─────────────────────────────────────┘
```

## How It Works

1. **Rust Code**: Written in `src/lib.rs` using `wasm-bindgen` for browser interop
2. **web-transport crate**: Automatically uses browser's native WebTransport API when compiled to WASM
3. **Compilation**: `wasm-pack` compiles Rust to WASM and generates JavaScript bindings
4. **Browser**: Loads WASM module and calls exported functions via JavaScript

## Files

- `src/lib.rs` - Rust WASM client code
- `Cargo.toml` - Rust dependencies and WASM configuration
- `index.html` - HTML page that loads and uses the WASM module
- `pkg/` - Generated WASM and JS files (after build)

## Comparison with JavaScript Client

| Feature | JavaScript Client | WASM Client |
|---------|------------------|-------------|
| Language | JavaScript | Rust |
| Compilation | None | Compiled to WASM |
| Type Safety | Runtime | Compile-time |
| Performance | Fast | Potentially faster |
| Bundle Size | Minimal | WASM + JS glue |
| API Used | Native WebTransport | web-transport crate |
