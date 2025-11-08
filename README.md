# WebTransport Test Project

A comprehensive test implementation of [wtransport](https://github.com/BiagioFesta/wtransport) demonstrating both JavaScript and WASM clients communicating with a Rust WebTransport server.

## Features

- **Rust Server** using wtransport 0.6
- **JavaScript Browser Client** using native WebTransport API
- **WASM Browser Client** compiled from Rust using web-transport crate
- **Bidirectional Streams** for reliable, ordered communication
- **Datagrams** for fast, unreliable communication
- **Certificate Pinning** for self-signed certificates (no browser warnings!)

## Quick Start

### 1. Generate Certificates

First, create the certificate configuration file:

```bash
cat > cert.conf << 'EOF'
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = State
L = City
O = Organization
CN = localhost

[v3_req]
keyUsage = keyEncipherment, dataEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
EOF
```

Generate ECDSA certificate (required for WebTransport):

```bash
# Generate ECDSA private key with prime256v1 curve
openssl ecparam -name prime256v1 -genkey -noout -out key.pem

# Generate certificate valid for 14 days (WebTransport requirement for cert pinning)
openssl req -new -x509 -key key.pem -out cert.pem -days 14 -config cert.conf -extensions v3_req
```

Get the certificate hash for client-side pinning:

```bash
openssl x509 -in cert.pem -outform der | openssl dgst -sha256 -binary | xxd -p -c 256
```

Update the hash in `client.html` and `wasm-client/src/lib.rs`.

### 2. Run the Server

```bash
cargo run
```

Server listens on:
- WebTransport: `https://localhost:8765`
- HTTP (for serving clients): `http://127.0.0.1:7654`

### 3. Test with JavaScript Client

Open your browser to `http://127.0.0.1:7654`

The JavaScript client (`client.html`) is served automatically by the server.

### 4. Test with WASM Client

```bash
cd wasm-client
wasm-pack build --target web
python3 -m http.server 9000
```

Open your browser to `http://localhost:9000`

## Clients Comparison

### JavaScript Client (`client.html`)

Uses the browser's native WebTransport API:
- Simpler implementation
- Direct browser API access
- Smaller bundle size

### WASM Client (`wasm-client/`)

Compiled from Rust using the [web-transport](https://crates.io/crates/web-transport) crate:
- Unified Rust API for native and web
- Type-safe implementation
- Share code between server and client
- Requires wasm-pack build step

## Browser Support

### Chromium/Chrome

WebTransport is supported natively. Just visit the client URL and click "Connect".

### Firefox

WebTransport support is experimental. Enable it in `about:config`:

1. Set `network.webtransport.enabled` to `true`
2. Set `network.webtransport.datagrams.enabled` to `true`

## Key Technical Details

### Certificate Requirements

WebTransport has strict certificate requirements:

- **Must use ECDSA** (not RSA)
- **Must use prime256v1 curve** (P-256 / secp256r1)
- For certificate pinning: **validity ≤14 days**
- Proper Subject Alternative Names (SAN) required

### WASM Client Architecture

The WASM client demonstrates important patterns for async Rust in WASM:

1. **Session Cloning**: The `web_transport::Session` type is cloneable, providing multiple handles to the same connection. This eliminates the need for `Arc<Mutex<>>` - just clone the session for each task.

2. **Concurrent Operations**: Separate clones handle:
   - Stream receiving (continuous reader task)
   - Datagram receiving (continuous reader task)
   - Stream sending (on-demand from user input)
   - Datagram sending (on-demand from user input)

3. **State Management**: Uses `thread_local` with `RefCell` for storing session and stream state in WASM's single-threaded environment.

4. **Configuration**: Requires `.with_unreliable(true)` on `ClientBuilder` to enable datagram support in WASM.

## Project Structure

```
wtransport-test/
├── src/
│   └── main.rs              # WebTransport server + HTTP file server
├── client.html              # JavaScript WebTransport client
├── wasm-client/             # WASM WebTransport client
│   ├── src/
│   │   └── lib.rs          # WASM client implementation
│   ├── index.html          # WASM loader page
│   ├── Cargo.toml          # WASM dependencies
│   └── .cargo/
│       └── config.toml     # Enable web_sys_unstable_apis
├── Cargo.toml              # Server dependencies
├── flake.nix               # Nix development environment
└── README.md
```

## Dependencies

### Server
- `wtransport = "0.6"` - WebTransport server implementation
- `tokio` - Async runtime
- `anyhow` - Error handling
- `tracing` - Logging

### WASM Client
- `web-transport = "0.9.7"` - Unified WebTransport API for native + WASM
- `wasm-bindgen` - Rust/JavaScript interop
- `web-sys` - Browser API bindings

## Common Issues

### Certificate Errors

If you see `cryptographic handshake failed: error 42/43`:
- Ensure you're using ECDSA (not RSA)
- Verify the certificate hash matches in client code
- Check certificate is valid and not expired

### Datagrams Not Working

- Ensure `ClientBuilder::new().with_unreliable(true)` is called (WASM)
- Check server is configured to accept datagrams
- Remember: datagrams can be dropped - this is expected behavior

### WASM Build Issues

If you see "WebTransport is not defined":
- Ensure `.cargo/config.toml` has `rustflags = ["--cfg=web_sys_unstable_apis"]`
- Check that all required web-sys features are enabled in Cargo.toml

## License

MIT OR Apache-2.0
