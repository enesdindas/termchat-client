use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::{
    api::RestClient,
    models::WsEnvelope,
    state::{AppState, ConversationKind, LoginField, Screen, SidebarItem},
};

/// Returns true if the application should quit.
pub async fn handle_key(
    key: KeyEvent,
    state: &mut AppState,
    ws_tx: &Option<mpsc::Sender<String>>,
    rest: &mut RestClient,
) -> bool {
    match state.screen {
        Screen::Login => handle_login_key(key, state, rest).await,
        Screen::Main => handle_main_key(key, state, ws_tx, rest).await,
    }
}

async fn handle_login_key(
    key: KeyEvent,
    state: &mut AppState,
    _rest: &mut RestClient,
) -> bool {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Tab => {
            state.login_field = if state.login_field == LoginField::Username {
                LoginField::Password
            } else {
                LoginField::Username
            };
            state.login_error = None;
        }
        // Register on Ctrl+R (must come before general Char match)
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.login_error = None;
            state.login_status = Some("Registering...".to_string());
        }
        KeyCode::Char(c) => match state.login_field {
            LoginField::Username => state.login_username.push(c),
            LoginField::Password => state.login_password.push(c),
        },
        KeyCode::Backspace => match state.login_field {
            LoginField::Username => { state.login_username.pop(); }
            LoginField::Password => { state.login_password.pop(); }
        },
        // Login on Enter
        KeyCode::Enter => {
            state.login_error = None;
            state.login_status = Some("Logging in...".to_string());
        }
        _ => {}
    }
    false
}

async fn handle_main_key(
    key: KeyEvent,
    state: &mut AppState,
    ws_tx: &Option<mpsc::Sender<String>>,
    _rest: &mut RestClient,
) -> bool {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Esc => {
            // Toggle focus to sidebar navigation
        }
        KeyCode::Enter => {
            let content = state.take_input();
            if content.is_empty() {
                return false;
            }
            if let Some(ws) = ws_tx {
                match &state.active_conversation {
                    Some(ConversationKind::Channel(channel_id)) => {
                        let env = WsEnvelope::new(
                            "message.send",
                            serde_json::json!({
                                "channel_id": channel_id,
                                "content": content,
                            }),
                        );
                        let _ = ws.send(serde_json::to_string(&env).unwrap()).await;
                    }
                    Some(ConversationKind::DM(partner_id)) => {
                        let env = WsEnvelope::new(
                            "dm.send",
                            serde_json::json!({
                                "recipient_id": partner_id,
                                "content": content,
                            }),
                        );
                        let _ = ws.send(serde_json::to_string(&env).unwrap()).await;
                    }
                    None => {
                        state.set_status("Select a channel or DM first");
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            state.push_input_char(c);
        }
        KeyCode::Backspace => {
            state.pop_input_char();
        }
        // Scroll chat up
        KeyCode::PageUp => {
            state.chat_scroll = state.chat_scroll.saturating_add(5);
        }
        // Scroll chat down
        KeyCode::PageDown => {
            state.chat_scroll = state.chat_scroll.saturating_sub(5);
        }
        // Navigate sidebar with Alt+Up/Down or Ctrl+Up/Down
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
            navigate_sidebar(state, -1);
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
            navigate_sidebar(state, 1);
        }
        _ => {}
    }
    false
}

fn navigate_sidebar(state: &mut AppState, delta: i32) {
    let items = state.sidebar_items();
    if items.is_empty() {
        return;
    }

    let current_idx = items.iter().position(|item| match item {
        SidebarItem::Channel(ch) => {
            state.active_conversation == Some(ConversationKind::Channel(ch.id))
        }
        SidebarItem::User(u) => {
            state.active_conversation == Some(ConversationKind::DM(u.id))
        }
    });

    let len = items.len() as i32;
    let next_idx = match current_idx {
        Some(i) => ((i as i32 + delta).rem_euclid(len)) as usize,
        None => 0,
    };

    match &items[next_idx] {
        SidebarItem::Channel(ch) => state.select_channel(ch.id),
        SidebarItem::User(u) => state.select_dm(u.id),
    }
}
