use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, window};
use web_transport::{ClientBuilder, SendStream, Session};

// Global state to store the session and send stream
// Session is cloneable and provides multiple handles to the same connection
// Use Rc<RefCell> for SendStream since stream operations are synchronous
struct ConnectionState {
    session: Option<Session>,
    send_stream: Option<Rc<RefCell<SendStream>>>,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            session: None,
            send_stream: None,
        }
    }
}

thread_local! {
    static CONNECTION: RefCell<ConnectionState> = RefCell::new(ConnectionState::new());
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console::log_1(&"WASM WebTransport client initialized".into());
}

#[wasm_bindgen]
pub async fn connect_to_server(url_str: String) -> Result<(), JsValue> {
    console::log_1(&format!("Connecting to: {}", url_str).into());

    // Parse the URL
    let url = url_str
        .parse()
        .map_err(|e| JsValue::from_str(&format!("Invalid URL: {:?}", e)))?;

    // Get the certificate hash (same as in client.html)
    let cert_hash_hex = "dbecff3c052db73b98936dc11ebce78bafe3d70044243835ed221f091ee0fea7";
    let cert_hash = hex_to_bytes(cert_hash_hex);

    // Build client with certificate pinning and enable unreliable transport (datagrams)
    let client = ClientBuilder::new()
        .with_unreliable(true)
        .with_server_certificate_hashes(vec![cert_hash])
        .map_err(|e| JsValue::from_str(&format!("Client build error: {:?}", e)))?;

    match client.connect(url).await {
        Ok(mut session) => {
            console::log_1(&"Connected successfully!".into());
            add_message("Connected successfully!", "system");

            // Open a bidirectional stream
            match session.open_bi().await {
                Ok((send_stream, mut recv_stream)) => {
                    console::log_1(&"Bidirectional stream opened".into());
                    add_message("Stream opened, ready to send/receive", "system");

                    // Clone session for datagram operations
                    // Session is cloneable and each clone is a handle to the same connection
                    let session_for_datagrams = session.clone();

                    // Store the session and send stream in global state
                    CONNECTION.with(|conn| {
                        let mut state = conn.borrow_mut();
                        state.session = Some(session);
                        state.send_stream = Some(Rc::new(RefCell::new(send_stream)));
                    });

                    // Spawn a task to continuously read from the stream
                    spawn_local(async move {
                        loop {
                            // Read up to 1024 bytes at a time
                            match recv_stream.read(1024).await {
                                Ok(Some(bytes)) => {
                                    let message = String::from_utf8_lossy(&bytes);
                                    console::log_1(&format!("Received [Stream]: {}", message).into());
                                    add_message(&format!("[Stream] {}", message), "received");
                                }
                                Ok(None) => {
                                    console::log_1(&"Stream closed by server".into());
                                    add_message("Stream closed by server", "system");
                                    break;
                                }
                                Err(e) => {
                                    console::error_1(&format!("Read error: {:?}", e).into());
                                    add_message(&format!("Read error: {:?}", e), "system");
                                    break;
                                }
                            }
                        }
                    });

                    // Spawn a task to receive datagrams
                    // Use the cloned session - no mutex needed!
                    spawn_local(async move {
                        let mut session_dg = session_for_datagrams;
                        loop {
                            match session_dg.recv_datagram().await {
                                Ok(bytes) => {
                                    let message = String::from_utf8_lossy(&bytes);
                                    console::log_1(&format!("Received [Datagram]: {}", message).into());
                                    add_message(&format!("[Datagram] {}", message), "received");
                                }
                                Err(e) => {
                                    console::error_1(&format!("Datagram recv error: {:?}", e).into());
                                    break;
                                }
                            }
                        }
                    });

                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Failed to open stream: {:?}", e);
                    console::error_1(&err_msg.clone().into());
                    add_message(&err_msg, "system");
                    Err(JsValue::from_str(&err_msg))
                }
            }
        }
        Err(e) => {
            let err_msg = format!("Connection failed: {:?}", e);
            console::error_1(&err_msg.clone().into());
            add_message(&err_msg, "system");
            Err(JsValue::from_str(&err_msg))
        }
    }
}

#[wasm_bindgen]
pub async fn send_message_stream(message: String) -> Result<(), JsValue> {
    console::log_1(&format!("Attempting to send: {}", message).into());

    // Get a cloned reference to the send stream
    let send_stream_rc = CONNECTION.with(|conn| {
        let state = conn.borrow();
        state.send_stream.clone()
    });

    match send_stream_rc {
        Some(stream_rc) => {
            // Now we can use the stream without holding the CONNECTION borrow
            let message_bytes = message.as_bytes().to_vec();

            let result = {
                let mut stream = stream_rc.borrow_mut();
                stream.write(&message_bytes).await
            };

            match result {
                Ok(_) => {
                    add_message(&message, "sent");
                    console::log_1(&"Message sent successfully".into());
                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Send error: {:?}", e);
                    console::error_1(&err_msg.clone().into());
                    add_message(&err_msg, "system");
                    Err(JsValue::from_str(&err_msg))
                }
            }
        }
        None => {
            let err_msg = "Not connected - no send stream available";
            console::error_1(&err_msg.into());
            add_message(err_msg, "system");
            Err(JsValue::from_str(err_msg))
        }
    }
}

#[wasm_bindgen]
pub async fn send_message_datagram(message: String) -> Result<(), JsValue> {
    console::log_1(&format!("Attempting to send datagram: {}", message).into());

    // Get a cloned session
    let session = CONNECTION.with(|conn| {
        let state = conn.borrow();
        state.session.clone()
    });

    match session {
        Some(mut sess) => {
            // Convert message to bytes
            let message_bytes = bytes::Bytes::from(message.as_bytes().to_vec());

            // Send the datagram - no mutex needed!
            match sess.send_datagram(message_bytes).await {
                Ok(_) => {
                    add_message(&format!("[Datagram] {}", message), "sent");
                    console::log_1(&"Datagram sent successfully".into());
                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Datagram send error: {:?}", e);
                    console::error_1(&err_msg.clone().into());
                    add_message(&err_msg, "system");
                    Err(JsValue::from_str(&err_msg))
                }
            }
        }
        None => {
            let err_msg = "Not connected - no session available";
            console::error_1(&err_msg.into());
            add_message(err_msg, "system");
            Err(JsValue::from_str(err_msg))
        }
    }
}

#[wasm_bindgen]
pub async fn disconnect() {
    console::log_1(&"Disconnecting...".into());

    // Get the session to close it
    let session = CONNECTION.with(|conn| {
        let mut state = conn.borrow_mut();

        // Clear the send stream
        state.send_stream = None;

        // Take the session
        state.session.take()
    });

    // Close the session if it exists
    if let Some(mut session) = session {
        session.close(0, "User requested disconnect");
    }

    add_message("Disconnected", "system");
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn add_message(text: &str, msg_type: &str) {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    if let Some(messages_div) = document.get_element_by_id("messages") {
        if let Some(message_div) = document.create_element("div").ok() {
            message_div.set_class_name(&format!("message {}", msg_type));
            message_div.set_text_content(Some(text));
            let _ = messages_div.append_child(&message_div);

            // Scroll to bottom
            if let Some(html_div) = messages_div.dyn_ref::<web_sys::HtmlElement>() {
                html_div.set_scroll_top(html_div.scroll_height());
            }
        }
    }
}

#[wasm_bindgen]
pub fn update_status(connected: bool) {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    if let Some(status_div) = document.get_element_by_id("status") {
        if connected {
            status_div.set_text_content(Some("Status: Connected"));
            status_div.set_class_name("status connected");
        } else {
            status_div.set_text_content(Some("Status: Disconnected"));
            status_div.set_class_name("status disconnected");
        }
    }

    // Enable/disable buttons
    if let Some(connect_btn) = document.get_element_by_id("connectBtn") {
        if let Some(btn) = connect_btn.dyn_ref::<web_sys::HtmlButtonElement>() {
            btn.set_disabled(connected);
        }
    }

    if let Some(disconnect_btn) = document.get_element_by_id("disconnectBtn") {
        if let Some(btn) = disconnect_btn.dyn_ref::<web_sys::HtmlButtonElement>() {
            btn.set_disabled(!connected);
        }
    }

    if let Some(send_btn) = document.get_element_by_id("sendStreamBtn") {
        if let Some(btn) = send_btn.dyn_ref::<web_sys::HtmlButtonElement>() {
            btn.set_disabled(!connected);
        }
    }

    if let Some(send_btn) = document.get_element_by_id("sendDatagramBtn") {
        if let Some(btn) = send_btn.dyn_ref::<web_sys::HtmlButtonElement>() {
            btn.set_disabled(!connected);
        }
    }
}
