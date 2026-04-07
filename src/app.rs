use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures_util::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::{
    api::{RestClient, WsConnection},
    config::Config,
    events::handle_key,
    models::WsEnvelope,
    state::{AppState, Screen},
    ui::layout,
};

pub struct App {
    config: Config,
}

impl App {
    pub fn new() -> Self {
        App { config: Config::load() }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let mut state = AppState::new();
        let mut rest = RestClient::new(self.config.server_url.clone());
        let mut ws_tx: Option<mpsc::Sender<String>> = None;
        let mut ws_rx: Option<mpsc::Receiver<WsEnvelope>> = None;

        // Try to restore saved token
        if let Some(token) = self.config.load_token() {
            rest.set_token(token.clone());
            // Verify token is still valid
            if let Ok(user) = rest.me().await {
                state.current_user = Some(user);
                state.screen = Screen::Main;
                self.init_main(&mut state, &mut rest, &token, &mut ws_tx, &mut ws_rx).await;
            }
        }

        let mut event_stream = EventStream::new();
        let mut tick = interval(Duration::from_millis(100));
        // Pending login action flags
        let mut pending_login = false;
        let mut pending_register = false;

        loop {
            terminal.draw(|f| layout::render(f, &state))?;

            tokio::select! {
                // Terminal events
                Some(Ok(event)) = event_stream.next() => {
                    if let Event::Key(key) = event {
                        // Intercept Enter/Ctrl+R before passing to handler to check login intent
                        if state.screen == Screen::Login {
                            match key.code {
                                KeyCode::Enter => { pending_login = true; }
                                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    pending_register = true;
                                }
                                _ => {}
                            }
                        }

                        let quit = handle_key(key, &mut state, &ws_tx, &mut rest).await;
                        if quit {
                            return Ok(());
                        }
                    }
                }

                // Incoming WebSocket messages
                Some(env) = async { if let Some(rx) = ws_rx.as_mut() { rx.recv().await } else { None } } => {
                    self.handle_ws_message(env, &mut state, &mut rest, &mut ws_tx, &mut ws_rx).await;
                }

                // Tick: process pending auth actions
                _ = tick.tick() => {
                    if pending_login {
                        pending_login = false;
                        state.login_error = None;
                        state.login_status = Some("Logging in...".to_string());
                        let username = state.login_username.clone();
                        let password = state.login_password.clone();
                        match rest.login(&username, &password).await {
                            Ok(data) => {
                                rest.set_token(data.token.clone());
                                let _ = self.config.save_token(&data.token);
                                state.current_user = Some(data.user);
                                state.login_status = None;
                                state.screen = Screen::Main;
                                self.init_main(&mut state, &mut rest, &data.token, &mut ws_tx, &mut ws_rx).await;
                            }
                            Err(e) => {
                                state.login_error = Some(format!("{}", e));
                                state.login_status = None;
                            }
                        }
                    }

                    if pending_register {
                        pending_register = false;
                        state.login_error = None;
                        state.login_status = Some("Registering...".to_string());
                        let username = state.login_username.clone();
                        let password = state.login_password.clone();
                        match rest.register(&username, &password).await {
                            Ok(_) => {
                                // Auto-login after register
                                match rest.login(&username, &password).await {
                                    Ok(data) => {
                                        rest.set_token(data.token.clone());
                                        let _ = self.config.save_token(&data.token);
                                        state.current_user = Some(data.user);
                                        state.login_status = None;
                                        state.screen = Screen::Main;
                                        self.init_main(&mut state, &mut rest, &data.token, &mut ws_tx, &mut ws_rx).await;
                                    }
                                    Err(e) => {
                                        state.login_error = Some(format!("{}", e));
                                        state.login_status = None;
                                    }
                                }
                            }
                            Err(e) => {
                                state.login_error = Some(format!("{}", e));
                                state.login_status = None;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init_main(
        &self,
        state: &mut AppState,
        rest: &mut RestClient,
        token: &str,
        ws_tx: &mut Option<mpsc::Sender<String>>,
        ws_rx: &mut Option<mpsc::Receiver<WsEnvelope>>,
    ) {
        // Load channels and users
        if let Ok(channels) = rest.list_channels().await {
            state.channels = channels;
        }
        if let Ok(users) = rest.list_users().await {
            state.users = users;
        }

        // Connect WebSocket
        let ws_url = self.config.ws_url();
        match WsConnection::connect(&ws_url, token).await {
            Ok((conn, rx)) => {
                // Subscribe to all known channels and DMs
                let channel_ids: Vec<i64> = state.channels.iter().map(|c| c.id).collect();
                let my_id = state.current_user.as_ref().map(|u| u.id).unwrap_or(0);
                let dm_user_ids: Vec<i64> = state
                    .users
                    .iter()
                    .filter(|u| u.id != my_id)
                    .map(|u| u.id)
                    .collect();

                let sub = WsEnvelope::new(
                    "subscribe",
                    serde_json::json!({
                        "channel_ids": channel_ids,
                        "dm_user_ids": dm_user_ids,
                    }),
                );
                let _ = conn.send(&sub).await;

                *ws_tx = Some(conn.sender);
                *ws_rx = Some(rx);

                // Select the first channel by default
                if let Some(first_ch) = state.channels.first() {
                    let id = first_ch.id;
                    state.select_channel(id);
                    // Load history
                    if let Ok(msgs) = rest.get_channel_messages(id, None).await {
                        for msg in msgs {
                            let queue = state.channel_messages.entry(id).or_default();
                            queue.push_back(msg);
                        }
                    }
                }
            }
            Err(e) => {
                state.set_status(format!("WS connect failed: {}", e));
            }
        }
    }

    async fn handle_ws_message(
        &self,
        env: WsEnvelope,
        state: &mut AppState,
        _rest: &mut RestClient,
        _ws_tx: &mut Option<mpsc::Sender<String>>,
        _ws_rx: &mut Option<mpsc::Receiver<WsEnvelope>>,
    ) {
        match env.event_type.as_str() {
            "message.new" => {
                if let Ok(msg) = serde_json::from_value::<crate::models::Message>(env.payload) {
                    state.add_channel_message(msg);
                }
            }
            "dm.new" => {
                if let Ok(dm) = serde_json::from_value::<crate::models::DirectMessage>(env.payload) {
                    let my_id = state.current_user.as_ref().map(|u| u.id).unwrap_or(0);
                    state.add_dm_message(dm, my_id);
                }
            }
            "error" => {
                if let Value::Object(map) = &env.payload {
                    if let Some(Value::String(msg)) = map.get("message") {
                        state.set_status(format!("Server error: {}", msg));
                    }
                }
            }
            _ => {}
        }
    }
}
