use std::collections::{HashMap, VecDeque};

use crate::models::{Channel, ChannelMember, DirectMessage, Message, User, WsEnvelope};

const MAX_MESSAGES: usize = 500;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Main,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationKind {
    Channel(i64),
    DM(i64), // partner user ID
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateChannelField {
    Name,
    Description,
    Privacy,
}

#[derive(Debug, Clone)]
pub enum Modal {
    None,
    CreateChannel {
        name: String,
        description: String,
        is_private: bool,
        field: CreateChannelField,
        error: Option<String>,
    },
    ChannelList {
        cursor: usize,
    },
    MembersList {
        channel_id: i64,
        members: Vec<ChannelMember>,
        loading: bool,
    },
    AddMember {
        channel_id: i64,
        username_input: String,
        error: Option<String>,
    },
    RemoveMember {
        channel_id: i64,
        members: Vec<ChannelMember>,
        cursor: usize,
        loading: bool,
    },
    ConfirmLogout,
}

impl Modal {
    pub fn is_open(&self) -> bool {
        !matches!(self, Modal::None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WsLifecycle {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
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

    // Message caches
    pub channel_messages: HashMap<i64, VecDeque<Message>>,
    pub dm_messages: HashMap<i64, VecDeque<DirectMessage>>, // keyed by partner ID

    // Input
    pub input_buffer: String,

    // Scroll offset for chat (lines from bottom)
    pub chat_scroll: u16,

    // Status / error bar
    pub status_message: Option<String>,
    pub ws_lifecycle: WsLifecycle,

    // Unread counts
    pub unread_channels: HashMap<i64, usize>,
    pub unread_dms: HashMap<i64, usize>,

    // Set when a DM is selected and history hasn't been loaded yet
    pub pending_dm_history: Option<i64>,

    // Modal state
    pub modal: Modal,

    // Pending async actions triggered from modals (drained in app tick)
    pub pending_create_channel: bool,
    pub pending_load_members: Option<i64>,
    pub pending_load_members_remove: Option<i64>,
    pub pending_add_member: Option<(i64, i64)>,
    pub pending_remove_member: Option<(i64, i64)>,
    pub pending_self_join: Option<i64>,
    pub pending_logout: bool,
    pub pending_ws_outbox: VecDeque<WsEnvelope>,
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
            channel_messages: HashMap::new(),
            dm_messages: HashMap::new(),
            input_buffer: String::new(),
            chat_scroll: 0,
            status_message: None,
            ws_lifecycle: WsLifecycle::Disconnected,
            unread_channels: HashMap::new(),
            unread_dms: HashMap::new(),
            pending_dm_history: None,
            modal: Modal::None,
            pending_create_channel: false,
            pending_load_members: None,
            pending_load_members_remove: None,
            pending_add_member: None,
            pending_remove_member: None,
            pending_self_join: None,
            pending_logout: false,
            pending_ws_outbox: VecDeque::new(),
        }
    }

    pub fn close_modal(&mut self) {
        self.modal = Modal::None;
    }

    pub fn open_create_channel(&mut self) {
        self.modal = Modal::CreateChannel {
            name: String::new(),
            description: String::new(),
            is_private: false,
            field: CreateChannelField::Name,
            error: None,
        };
    }

    pub fn open_channel_list(&mut self) {
        self.modal = Modal::ChannelList { cursor: 0 };
    }

    pub fn open_members_list(&mut self, channel_id: i64) {
        self.modal = Modal::MembersList {
            channel_id,
            members: Vec::new(),
            loading: true,
        };
        self.pending_load_members = Some(channel_id);
    }

    pub fn open_add_member(&mut self, channel_id: i64) {
        self.modal = Modal::AddMember {
            channel_id,
            username_input: String::new(),
            error: None,
        };
    }

    pub fn open_remove_member(&mut self, channel_id: i64) {
        self.modal = Modal::RemoveMember {
            channel_id,
            members: Vec::new(),
            cursor: 0,
            loading: true,
        };
        self.pending_load_members_remove = Some(channel_id);
    }

    pub fn open_confirm_logout(&mut self) {
        self.modal = Modal::ConfirmLogout;
    }

    pub fn active_channel_id(&self) -> Option<i64> {
        match self.active_conversation {
            Some(ConversationKind::Channel(id)) => Some(id),
            _ => None,
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
