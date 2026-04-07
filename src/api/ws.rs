use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::models::WsEnvelope;

pub struct WsConnection {
    pub sender: mpsc::Sender<String>,
}

impl WsConnection {
    /// Connect to the WebSocket server. Returns the connection handle and a receiver
    /// for incoming envelopes.
    pub async fn connect(
        url: &str,
        token: &str,
    ) -> Result<(Self, mpsc::Receiver<WsEnvelope>)> {
        let ws_url = format!("{}?token={}", url, token);
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let (incoming_tx, incoming_rx) = mpsc::channel::<WsEnvelope>(256);
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(256);

        // Reader task
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(env) = serde_json::from_str::<WsEnvelope>(&text) {
                        if incoming_tx.send(env).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Writer task
        tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if write.send(Message::Text(msg.into())).await.is_err() {
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
