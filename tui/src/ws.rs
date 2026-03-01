use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

use crate::models::WsMessage;

pub async fn start_ws_client(
    tx: mpsc::UnboundedSender<WsMessage>,
    mut rx_out: mpsc::UnboundedReceiver<String>,
) {
    let url = Url::parse("ws://localhost:8000/ws/rpc").unwrap();
    
    // Retry loop — Python backend may still be warming up
    let ws_stream = loop {
        match connect_async(url.clone()).await {
            Ok((stream, _)) => break stream,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Spawn task to read from WS and send to MPSC
    let read_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse into our strict Data Contract
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        if tx.send(ws_msg).is_err() {
                            break; // Receiver dropped
                        }
                    } else {
                        // Helpful for debugging contract mismatches
                        eprintln!("Failed to parse WS message: {}", text);
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break, // Connection closed or error
                _ => {}          // Ignore other msg types for now
            }
        }
    });

    // Spawn task to read from MPSC out queue and send to WS
    let write_task = tokio::spawn(async move {
        while let Some(out_msg) = rx_out.recv().await {
            // In a real app we might serialize a Prompt command, but for now simple String JSON
            // `{"prompt": "my message"}` as expected by Python server
            let json_msg = serde_json::json!({ "prompt": out_msg });
            if write
                .send(Message::Text(json_msg.to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Wait for either to finish (error or close)
    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }
}
