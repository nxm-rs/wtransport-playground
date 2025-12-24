gen-key:
	# Generate ECDSA cert (WebTransport requires ECDSA + prime256v1 + ≤14 days)
	openssl ecparam -name prime256v1 -genkey -noout -out key.pem
gen-cert:
	openssl req -new -x509 -key key.pem -out cert.pem -days 14 -config cert.conf -extensions v3_req
	# Get cert hash and update in client.html and wasm-client/src/lib.rs
	openssl x509 -in cert.pem -outform der | openssl dgst -sha256 -binary | xxd -p -c 256
read-key:
	openssl pkey -in key.pem -text -noout
read-cert:
	openssl x509 -in cert.pem -text -noout
