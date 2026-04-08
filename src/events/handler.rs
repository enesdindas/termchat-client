use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::{
    api::RestClient,
    models::WsEnvelope,
    state::{AppState, ConversationKind, CreateChannelField, LoginField, Modal, Screen, SidebarItem},
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
    // Modal captures all input first
    if state.modal.is_open() {
        handle_modal_key(key, state);
        return false;
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.open_create_channel();
        }
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.open_channel_list();
        }
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(id) = state.active_channel_id() {
                state.open_members_list(id);
            } else {
                state.set_status("Select a channel first");
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(id) = state.active_channel_id() {
                state.open_add_member(id);
            } else {
                state.set_status("Select a channel first");
            }
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(id) = state.active_channel_id() {
                state.open_remove_member(id);
            } else {
                state.set_status("Select a channel first");
            }
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(id) = state.active_channel_id() {
                state.pending_self_join = Some(id);
            } else {
                state.set_status("Select a channel first");
            }
        }
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.open_confirm_logout();
        }
        KeyCode::Enter => {
            let content = state.take_input();
            if content.is_empty() {
                return false;
            }
            let env = match &state.active_conversation {
                Some(ConversationKind::Channel(channel_id)) => WsEnvelope::new(
                    "message.send",
                    serde_json::json!({
                        "channel_id": channel_id,
                        "content": content,
                    }),
                ),
                Some(ConversationKind::DM(partner_id)) => WsEnvelope::new(
                    "dm.send",
                    serde_json::json!({
                        "recipient_id": partner_id,
                        "content": content,
                    }),
                ),
                None => {
                    state.set_status("Select a channel or DM first");
                    return false;
                }
            };

            if let Some(ws) = ws_tx {
                let payload = serde_json::to_string(&env).unwrap();
                if ws.send(payload).await.is_err() {
                    state.pending_ws_outbox.push_back(env);
                    state.set_status("Connection issue: message queued for retry");
                }
            } else {
                state.pending_ws_outbox.push_back(env);
                state.set_status("Offline: message queued for retry");
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
        // Up/Down navigate sidebar when input is empty; Alt+Up/Down always navigate
        KeyCode::Up if state.input_buffer.is_empty() || key.modifiers.contains(KeyModifiers::ALT) => {
            navigate_sidebar(state, -1);
        }
        KeyCode::Down if state.input_buffer.is_empty() || key.modifiers.contains(KeyModifiers::ALT) => {
            navigate_sidebar(state, 1);
        }
        _ => {}
    }
    false
}

fn handle_modal_key(key: KeyEvent, state: &mut AppState) {
    // Universal cancel
    if key.code == KeyCode::Esc {
        state.close_modal();
        return;
    }

    // Take ownership of the modal so we can mutate freely without borrowing state.
    let modal = std::mem::replace(&mut state.modal, Modal::None);
    match modal {
        Modal::None => {}
        Modal::CreateChannel {
            mut name,
            mut description,
            mut is_private,
            mut field,
            mut error,
        } => match key.code {
            KeyCode::Tab => {
                field = match field {
                    CreateChannelField::Name => CreateChannelField::Description,
                    CreateChannelField::Description => CreateChannelField::Privacy,
                    CreateChannelField::Privacy => CreateChannelField::Name,
                };
                state.modal = Modal::CreateChannel { name, description, is_private, field, error };
            }
            KeyCode::Char(' ') if field == CreateChannelField::Privacy => {
                is_private = !is_private;
                state.modal = Modal::CreateChannel { name, description, is_private, field, error };
            }
            KeyCode::Char(c) => {
                match field {
                    CreateChannelField::Name => name.push(c),
                    CreateChannelField::Description => description.push(c),
                    CreateChannelField::Privacy => {}
                }
                state.modal = Modal::CreateChannel { name, description, is_private, field, error };
            }
            KeyCode::Backspace => {
                match field {
                    CreateChannelField::Name => { name.pop(); }
                    CreateChannelField::Description => { description.pop(); }
                    CreateChannelField::Privacy => {}
                }
                state.modal = Modal::CreateChannel { name, description, is_private, field, error };
            }
            KeyCode::Enter => {
                if name.trim().is_empty() {
                    error = Some("name required".to_string());
                    state.modal = Modal::CreateChannel { name, description, is_private, field, error };
                } else {
                    state.pending_create_channel = true;
                    state.modal = Modal::CreateChannel { name, description, is_private, field, error };
                }
            }
            _ => {
                state.modal = Modal::CreateChannel { name, description, is_private, field, error };
            }
        },
        Modal::ChannelList { mut cursor } => {
            let len = state.channels.len();
            match key.code {
                KeyCode::Up => {
                    if len > 0 {
                        cursor = if cursor == 0 { len - 1 } else { cursor - 1 };
                    }
                    state.modal = Modal::ChannelList { cursor };
                }
                KeyCode::Down => {
                    if len > 0 {
                        cursor = (cursor + 1) % len;
                    }
                    state.modal = Modal::ChannelList { cursor };
                }
                KeyCode::Enter => {
                    if let Some(ch) = state.channels.get(cursor).cloned() {
                        state.select_channel(ch.id);
                    }
                    state.close_modal();
                }
                _ => {
                    state.modal = Modal::ChannelList { cursor };
                }
            }
        }
        Modal::MembersList { channel_id, members, loading } => {
            // Read-only; any non-Esc key is ignored
            state.modal = Modal::MembersList { channel_id, members, loading };
        }
        Modal::AddMember { channel_id, mut username_input, mut error } => match key.code {
            KeyCode::Char(c) => {
                username_input.push(c);
                state.modal = Modal::AddMember { channel_id, username_input, error };
            }
            KeyCode::Backspace => {
                username_input.pop();
                state.modal = Modal::AddMember { channel_id, username_input, error };
            }
            KeyCode::Enter => {
                let target = state
                    .users
                    .iter()
                    .find(|u| u.username.eq_ignore_ascii_case(username_input.trim()));
                if let Some(u) = target {
                    state.pending_add_member = Some((channel_id, u.id));
                    state.modal = Modal::AddMember { channel_id, username_input, error };
                } else {
                    error = Some(format!("user '{}' not found", username_input.trim()));
                    state.modal = Modal::AddMember { channel_id, username_input, error };
                }
            }
            _ => {
                state.modal = Modal::AddMember { channel_id, username_input, error };
            }
        },
        Modal::RemoveMember { channel_id, members, mut cursor, loading } => match key.code {
            KeyCode::Up => {
                if !members.is_empty() {
                    cursor = if cursor == 0 { members.len() - 1 } else { cursor - 1 };
                }
                state.modal = Modal::RemoveMember { channel_id, members, cursor, loading };
            }
            KeyCode::Down => {
                if !members.is_empty() {
                    cursor = (cursor + 1) % members.len();
                }
                state.modal = Modal::RemoveMember { channel_id, members, cursor, loading };
            }
            KeyCode::Enter => {
                if let Some(m) = members.get(cursor) {
                    state.pending_remove_member = Some((channel_id, m.user_id));
                }
                state.modal = Modal::RemoveMember { channel_id, members, cursor, loading };
            }
            _ => {
                state.modal = Modal::RemoveMember { channel_id, members, cursor, loading };
            }
        },
        Modal::ConfirmLogout => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                state.pending_logout = true;
                state.close_modal();
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                state.close_modal();
            }
            _ => {
                state.modal = Modal::ConfirmLogout;
            }
        },
    }
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
