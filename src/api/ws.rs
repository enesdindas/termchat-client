use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::models::WsEnvelope;

#[derive(Debug)]
pub enum WsEvent {
    Envelope(WsEnvelope),
    Disconnected(String),
}

pub struct WsConnection {
    pub sender: mpsc::Sender<String>,
}

impl WsConnection {
    /// Connect to the WebSocket server. Returns the connection handle and a receiver
    /// for incoming envelopes.
    pub async fn connect(
        url: &str,
        token: &str,
    ) -> Result<(Self, mpsc::Receiver<WsEvent>)> {
        let ws_url = format!("{}?token={}", url, token);
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let (incoming_tx, incoming_rx) = mpsc::channel::<WsEvent>(256);
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(256);
        let closed = Arc::new(AtomicBool::new(false));

        // Reader task
        let reader_tx = incoming_tx.clone();
        let reader_closed = Arc::clone(&closed);
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(env) = serde_json::from_str::<WsEnvelope>(&text) {
                        if reader_tx.send(WsEvent::Envelope(env)).await.is_err() {
                            break;
                        }
                    }
                }
            }
            if !reader_closed.swap(true, Ordering::SeqCst) {
                let _ = reader_tx
                    .send(WsEvent::Disconnected("websocket disconnected".to_string()))
                    .await;
            }
        });

        // Writer task
        let writer_tx = incoming_tx.clone();
        let writer_closed = Arc::clone(&closed);
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Err(e) = write.send(Message::Text(msg)).await {
                    if !writer_closed.swap(true, Ordering::SeqCst) {
                        let _ = writer_tx
                            .send(WsEvent::Disconnected(format!("websocket write failed: {}", e)))
                            .await;
                    }
                    break;
                }
            }
        });

        Ok((WsConnection { sender: outgoing_tx }, incoming_rx))
    }

    /// Send a typed envelope over the WebSocket.
    pub async fn send(&self, env: &WsEnvelope) -> Result<()> {
        let text = serde_json::to_string(env)?;
        self.sender.send(text).await?;
        Ok(())
    }
}
