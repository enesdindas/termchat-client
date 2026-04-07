use std::collections::{HashMap, VecDeque};

use crate::models::{Channel, DirectMessage, Message, User};

const MAX_MESSAGES: usize = 500;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Main,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePane {
    Sidebar,
    Chat,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationKind {
    Channel(i64),
    DM(i64), // partner user ID
}

pub struct AppState {
    pub screen: Screen,
    pub current_user: Option<User>,

    // Login form
    pub login_username: String,
    pub login_password: String,
    pub login_field: LoginField,
    pub login_error: Option<String>,
    pub login_status: Option<String>, // e.g. "Logging in..."

    // Main screen
    pub channels: Vec<Channel>,
    pub users: Vec<User>, // all users for DM
    pub active_conversation: Option<ConversationKind>,
    pub active_pane: ActivePane,

    // Message caches
    pub channel_messages: HashMap<i64, VecDeque<Message>>,
    pub dm_messages: HashMap<i64, VecDeque<DirectMessage>>, // keyed by partner ID

    // Input
    pub input_buffer: String,

    // Scroll offset for chat (lines from bottom)
    pub chat_scroll: u16,

    // Status / error bar
    pub status_message: Option<String>,

    // Unread counts
    pub unread_channels: HashMap<i64, usize>,
    pub unread_dms: HashMap<i64, usize>,

    // Set when a DM is selected and history hasn't been loaded yet
    pub pending_dm_history: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginField {
    Username,
    Password,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Screen::Login,
            current_user: None,
            login_username: String::new(),
            login_password: String::new(),
            login_field: LoginField::Username,
            login_error: None,
            login_status: None,
            channels: Vec::new(),
            users: Vec::new(),
            active_conversation: None,
            active_pane: ActivePane::Input,
            channel_messages: HashMap::new(),
            dm_messages: HashMap::new(),
            input_buffer: String::new(),
            chat_scroll: 0,
            status_message: None,
            unread_channels: HashMap::new(),
            unread_dms: HashMap::new(),
            pending_dm_history: None,
        }
    }

    pub fn select_channel(&mut self, channel_id: i64) {
        self.active_conversation = Some(ConversationKind::Channel(channel_id));
        self.chat_scroll = 0;
        self.unread_channels.remove(&channel_id);
    }

    pub fn select_dm(&mut self, partner_id: i64) {
        self.active_conversation = Some(ConversationKind::DM(partner_id));
        self.chat_scroll = 0;
        self.unread_dms.remove(&partner_id);
        // Trigger history load if not yet cached
        if !self.dm_messages.contains_key(&partner_id) {
            self.pending_dm_history = Some(partner_id);
        }
    }

    pub fn add_channel_message(&mut self, msg: Message) {
        let is_active = self.active_conversation == Some(ConversationKind::Channel(msg.channel_id));
        if !is_active {
            *self.unread_channels.entry(msg.channel_id).or_insert(0) += 1;
        }
        let queue = self.channel_messages.entry(msg.channel_id).or_default();
        queue.push_back(msg);
        if queue.len() > MAX_MESSAGES {
            queue.pop_front();
        }
        if is_active {
            self.chat_scroll = 0;
        }
    }

    pub fn add_dm_message(&mut self, dm: DirectMessage, my_user_id: i64) {
        let partner_id = if dm.sender_id == my_user_id {
            dm.recipient_id
        } else {
            dm.sender_id
        };
        let is_active = self.active_conversation == Some(ConversationKind::DM(partner_id));
        if !is_active {
            *self.unread_dms.entry(partner_id).or_insert(0) += 1;
        }
        let queue = self.dm_messages.entry(partner_id).or_default();
        queue.push_back(dm);
        if queue.len() > MAX_MESSAGES {
            queue.pop_front();
        }
        if is_active {
            self.chat_scroll = 0;
        }
    }

    pub fn push_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn pop_input_char(&mut self) {
        self.input_buffer.pop();
    }

    pub fn take_input(&mut self) -> String {
        std::mem::take(&mut self.input_buffer)
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Returns sidebar items: channels then DM users
    pub fn sidebar_items(&self) -> Vec<SidebarItem> {
        let mut items = Vec::new();
        for ch in &self.channels {
            items.push(SidebarItem::Channel(ch.clone()));
        }
        for u in &self.users {
            if let Some(me) = &self.current_user {
                if u.id != me.id {
                    items.push(SidebarItem::User(u.clone()));
                }
            }
        }
        items
    }
}

#[derive(Debug, Clone)]
pub enum SidebarItem {
    Channel(Channel),
    User(crate::models::User),
}
