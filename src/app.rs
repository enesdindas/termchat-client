use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures_util::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::{
    api::{RestClient, WsConnection, WsEvent},
    config::Config,
    events::handle_key,
    models::WsEnvelope,
    state::{AppState, Modal, Screen, WsLifecycle},
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
        let mut ws_rx: Option<mpsc::Receiver<WsEvent>> = None;
        let mut session_token: Option<String> = None;
        let mut reconnect_attempts: u8 = 0;
        let mut reconnect_at: Option<Instant> = None;

        // Try to restore saved token
        if let Some(token) = self.config.load_token() {
            rest.set_token(token.clone());
            // Verify token is still valid
            if let Ok(user) = rest.me().await {
                session_token = Some(token.clone());
                state.current_user = Some(user);
                state.screen = Screen::Main;
                if !self.init_main(&mut state, &mut rest, &token, &mut ws_tx, &mut ws_rx).await {
                    state.ws_lifecycle = WsLifecycle::Reconnecting;
                    reconnect_at = Some(Instant::now() + Duration::from_secs(1));
                    reconnect_attempts = 1;
                }
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

                // Incoming WebSocket events
                Some(event) = async { if let Some(rx) = ws_rx.as_mut() { rx.recv().await } else { None } } => {
                    match event {
                        WsEvent::Envelope(env) => {
                            self.handle_ws_message(env, &mut state).await;
                        }
                        WsEvent::Disconnected(reason) => {
                            ws_tx = None;
                            ws_rx = None;
                            if state.screen == Screen::Main && session_token.is_some() {
                                state.ws_lifecycle = WsLifecycle::Reconnecting;
                                state.set_status(format!("Connection lost: {}", reason));
                                if reconnect_attempts < 5 {
                                    reconnect_attempts += 1;
                                    let wait_secs = 2u64.pow(reconnect_attempts.saturating_sub(1) as u32);
                                    reconnect_at = Some(Instant::now() + Duration::from_secs(wait_secs));
                                } else {
                                    state.ws_lifecycle = WsLifecycle::Disconnected;
                                    state.set_status("Disconnected: retry limit reached");
                                }
                            }
                        }
                    }
                }

                // Tick: process pending auth actions and lazy history loads
                _ = tick.tick() => {
                    if let Some(tx) = ws_tx.as_ref() {
                        while let Some(env) = state.pending_ws_outbox.front().cloned() {
                            let payload = serde_json::to_string(&env).unwrap();
                            if tx.send(payload).await.is_ok() {
                                state.pending_ws_outbox.pop_front();
                            } else {
                                break;
                            }
                        }
                    }

                    if let Some(partner_id) = state.pending_dm_history.take() {
                        if let Ok(msgs) = rest.get_dm_history(partner_id, None).await {
                            let queue = state.dm_messages.entry(partner_id).or_default();
                            for dm in msgs {
                                queue.push_back(dm);
                            }
                            if state.active_conversation == Some(crate::state::ConversationKind::DM(partner_id)) {
                                state.chat_scroll = 0;
                            }
                        }
                    }

                    if pending_login {
                        pending_login = false;
                        state.login_error = None;
                        state.login_status = Some("Logging in...".to_string());
                        let username = state.login_username.clone();
                        let password = state.login_password.clone();
                        match rest.login(&username, &password).await {
                            Ok(data) => {
                                rest.set_token(data.token.clone());
                                session_token = Some(data.token.clone());
                                let _ = self.config.save_token(&data.token);
                                state.current_user = Some(data.user);
                                state.login_status = None;
                                state.screen = Screen::Main;
                                if self.init_main(&mut state, &mut rest, &data.token, &mut ws_tx, &mut ws_rx).await {
                                    reconnect_attempts = 0;
                                    reconnect_at = None;
                                }
                            }
                            Err(e) => {
                                state.login_error = Some(format!("{}", e));
                                state.login_status = None;
                            }
                        }
                    }

                    self.process_pending_modal_actions(&mut state, &mut rest, &mut ws_tx, &mut ws_rx).await;
                    if state.pending_logout {
                        session_token = None;
                        reconnect_attempts = 0;
                        reconnect_at = None;
                    }

                    if state.screen == Screen::Main
                        && ws_tx.is_none()
                        && session_token.is_some()
                        && reconnect_attempts > 0
                    {
                        if let Some(deadline) = reconnect_at {
                            if Instant::now() >= deadline {
                                if let Some(token) = session_token.as_ref() {
                                    state.ws_lifecycle = WsLifecycle::Reconnecting;
                                    if self.connect_ws(&mut state, token, &mut ws_tx, &mut ws_rx).await.is_ok() {
                                        reconnect_attempts = 0;
                                        reconnect_at = None;
                                        state.set_status("Reconnected");
                                    } else if reconnect_attempts < 5 {
                                        reconnect_attempts += 1;
                                        let wait_secs = 2u64.pow(reconnect_attempts.saturating_sub(1) as u32);
                                        reconnect_at = Some(Instant::now() + Duration::from_secs(wait_secs));
                                    } else {
                                        state.ws_lifecycle = WsLifecycle::Disconnected;
                                        state.set_status("Disconnected: retry limit reached");
                                    }
                                }
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
                                        session_token = Some(data.token.clone());
                                        let _ = self.config.save_token(&data.token);
                                        state.current_user = Some(data.user);
                                        state.login_status = None;
                                        state.screen = Screen::Main;
                                        if self.init_main(&mut state, &mut rest, &data.token, &mut ws_tx, &mut ws_rx).await {
                                            reconnect_attempts = 0;
                                            reconnect_at = None;
                                        }
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
        ws_rx: &mut Option<mpsc::Receiver<WsEvent>>,
    ) -> bool {
        // Load channels and users
        if let Ok(channels) = rest.list_channels().await {
            state.channels = channels;
        }
        if let Ok(users) = rest.list_users().await {
            state.users = users;
        }

        if let Err(e) = self.connect_ws(state, token, ws_tx, ws_rx).await {
            state.set_status(format!("WS connect failed: {}", e));
            return false;
        }

        // Load history for all channels once during initial session entry.
        let channel_ids: Vec<i64> = state.channels.iter().map(|c| c.id).collect();
        for id in channel_ids {
            if state.channel_messages.contains_key(&id) {
                continue;
            }
            if let Ok(msgs) = rest.get_channel_messages(id, None).await {
                let queue = state.channel_messages.entry(id).or_default();
                for msg in msgs {
                    queue.push_back(msg);
                }
            }
        }
        // Select the first channel by default only if nothing is selected.
        if state.active_conversation.is_none() {
            if let Some(first_ch) = state.channels.first() {
                let id = first_ch.id;
                state.select_channel(id);
            }
        }
        true
    }

    async fn connect_ws(
        &self,
        state: &mut AppState,
        token: &str,
        ws_tx: &mut Option<mpsc::Sender<String>>,
        ws_rx: &mut Option<mpsc::Receiver<WsEvent>>,
    ) -> Result<()> {
        state.ws_lifecycle = WsLifecycle::Connecting;
        let ws_url = self.config.ws_url();
        let (conn, rx) = WsConnection::connect(&ws_url, token).await?;

        // Subscribe to all known channels and DMs.
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
        conn.send(&sub).await?;

        *ws_tx = Some(conn.sender);
        *ws_rx = Some(rx);
        state.ws_lifecycle = WsLifecycle::Connected;
        Ok(())
    }

    async fn process_pending_modal_actions(
        &self,
        state: &mut AppState,
        rest: &mut RestClient,
        ws_tx: &mut Option<mpsc::Sender<String>>,
        ws_rx: &mut Option<mpsc::Receiver<WsEvent>>,
    ) {
        // Create channel
        if state.pending_create_channel {
            state.pending_create_channel = false;
            if let Modal::CreateChannel { name, description, is_private, field, .. } = state.modal.clone() {
                match rest.create_channel(name.trim(), description.trim(), is_private).await {
                    Ok(ch) => {
                        let id = ch.id;
                        state.channels.push(ch);
                        state.channels.sort_by(|a, b| a.name.cmp(&b.name));
                        state.set_status(format!("Created channel #{}", name));
                        state.close_modal();
                        // Subscribe to new channel via existing WS
                        if let Some(tx) = ws_tx.as_ref() {
                            let env = WsEnvelope::new(
                                "subscribe",
                                serde_json::json!({ "channel_ids": [id], "dm_user_ids": Vec::<i64>::new() }),
                            );
                            let _ = tx.send(serde_json::to_string(&env).unwrap()).await;
                        }
                        state.select_channel(id);
                    }
                    Err(e) => {
                        state.modal = Modal::CreateChannel {
                            name,
                            description,
                            is_private,
                            field,
                            error: Some(format!("{}", e)),
                        };
                    }
                }
            }
        }

        // Load members for MembersList modal
        if let Some(channel_id) = state.pending_load_members.take() {
            match rest.list_members(channel_id).await {
                Ok(members) => {
                    if let Modal::MembersList { channel_id: cid, .. } = &state.modal {
                        if *cid == channel_id {
                            state.modal = Modal::MembersList {
                                channel_id,
                                members,
                                loading: false,
                            };
                        }
                    }
                }
                Err(e) => {
                    state.set_status(format!("Members load failed: {}", e));
                    state.close_modal();
                }
            }
        }

        // Load members for RemoveMember modal
        if let Some(channel_id) = state.pending_load_members_remove.take() {
            match rest.list_members(channel_id).await {
                Ok(members) => {
                    if let Modal::RemoveMember { channel_id: cid, .. } = &state.modal {
                        if *cid == channel_id {
                            state.modal = Modal::RemoveMember {
                                channel_id,
                                members,
                                cursor: 0,
                                loading: false,
                            };
                        }
                    }
                }
                Err(e) => {
                    state.set_status(format!("Members load failed: {}", e));
                    state.close_modal();
                }
            }
        }

        // Add member
        if let Some((channel_id, user_id)) = state.pending_add_member.take() {
            match rest.add_member(channel_id, user_id).await {
                Ok(_) => {
                    state.set_status("User added");
                    state.close_modal();
                }
                Err(e) => {
                    if let Modal::AddMember { username_input, .. } = state.modal.clone() {
                        state.modal = Modal::AddMember {
                            channel_id,
                            username_input,
                            error: Some(format!("{}", e)),
                        };
                    }
                }
            }
        }

        // Remove member
        if let Some((channel_id, user_id)) = state.pending_remove_member.take() {
            match rest.remove_member(channel_id, user_id).await {
                Ok(_) => {
                    state.set_status("User removed");
                    // Refresh the member list in the modal
                    if let Ok(members) = rest.list_members(channel_id).await {
                        if let Modal::RemoveMember { cursor, .. } = &state.modal {
                            let new_cursor = (*cursor).min(members.len().saturating_sub(1));
                            state.modal = Modal::RemoveMember {
                                channel_id,
                                members,
                                cursor: new_cursor,
                                loading: false,
                            };
                        }
                    }
                }
                Err(e) => {
                    state.set_status(format!("Remove failed: {}", e));
                }
            }
        }

        // Self-join (Ctrl+J)
        if let Some(channel_id) = state.pending_self_join.take() {
            match rest.join_channel(channel_id).await {
                Ok(_) => state.set_status("Joined channel"),
                Err(e) => state.set_status(format!("Join failed: {}", e)),
            }
        }

        // Logout
        if state.pending_logout {
            state.pending_logout = false;
            let _ = self.config.delete_token();
            *ws_tx = None;
            *ws_rx = None;
            *state = AppState::new();
            state.set_status("Logged out");
        }
    }

    async fn handle_ws_message(
        &self,
        env: WsEnvelope,
        state: &mut AppState,
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
