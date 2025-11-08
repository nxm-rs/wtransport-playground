# wtransport-playground

Playground for testing [wtransport](https://github.com/BiagioFesta/wtransport) with JavaScript and WASM clients.

## Features

- Rust WebTransport server (wtransport 0.6)
- JavaScript client (native browser API)
- WASM client (compiled from Rust)
- Bidirectional streams and datagrams
- Certificate pinning for self-signed certs

## Quick Start

### 1. Generate Certificates

```bash
# Create cert config
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

# Generate ECDSA cert (WebTransport requires ECDSA + prime256v1 + ≤14 days)
openssl ecparam -name prime256v1 -genkey -noout -out key.pem
openssl req -new -x509 -key key.pem -out cert.pem -days 14 -config cert.conf -extensions v3_req

# Get cert hash and update in client.html and wasm-client/src/lib.rs
openssl x509 -in cert.pem -outform der | openssl dgst -sha256 -binary | xxd -p -c 256
```

### 2. Run Server

```bash
cargo run
```

- WebTransport: `https://localhost:8765`
- HTTP: `http://127.0.0.1:7654`

### 3. Test Clients

**JavaScript:**
```bash
# Open http://127.0.0.1:7654 (served by the server)
```

**WASM:**
```bash
cd wasm-client
wasm-pack build --target web
python3 -m http.server 9000
# Open http://localhost:9000
```

## Architecture Notes

### WASM Client Pattern

Key learnings for async Rust in WASM:

- **Session Cloning**: `web_transport::Session` is cloneable - no `Arc<Mutex<>>` needed
- **Concurrent Ops**: Clone session for each send/receive task
- **State**: Use `thread_local` + `RefCell` for WASM's single-threaded environment
- **Datagrams**: Must call `.with_unreliable(true)` on `ClientBuilder`

### Certificate Requirements

- ECDSA only (not RSA)
- prime256v1 curve (P-256)
- ≤14 days validity for cert pinning
- Proper SAN extensions

## Browser Support

- **Chrome/Chromium**: Native support
- **Firefox**: Experimental (enable in `about:config`)

## License

MIT OR Apache-2.0
