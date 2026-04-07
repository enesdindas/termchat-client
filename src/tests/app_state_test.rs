#[cfg(test)]
mod tests {
    use crate::{
        models::{Channel, DirectMessage, Message, User},
        state::{AppState, ConversationKind, Screen},
    };

    fn make_user(id: i64, username: &str) -> User {
        User { id, username: username.to_string(), created_at: "2026-01-01T00:00:00Z".to_string() }
    }

    fn make_message(id: i64, channel_id: i64, content: &str) -> Message {
        Message {
            id,
            channel_id,
            author_id: 1,
            author_username: "alice".to_string(),
            content: content.to_string(),
            created_at: "2026-04-07T12:00:00Z".to_string(),
        }
    }

    fn make_dm(id: i64, sender_id: i64, recipient_id: i64, content: &str) -> DirectMessage {
        DirectMessage {
            id,
            sender_id,
            sender_username: "alice".to_string(),
            recipient_id,
            content: content.to_string(),
            created_at: "2026-04-07T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_initial_state() {
        let state = AppState::new();
        assert_eq!(state.screen, Screen::Login);
        assert!(state.current_user.is_none());
        assert!(state.channels.is_empty());
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_select_channel_clears_unread() {
        let mut state = AppState::new();
        *state.unread_channels.entry(1).or_insert(0) = 3;
        state.select_channel(1);
        assert_eq!(state.active_conversation, Some(ConversationKind::Channel(1)));
        assert_eq!(state.unread_channels.get(&1).copied().unwrap_or(0), 0);
        assert_eq!(state.chat_scroll, 0);
    }

    #[test]
    fn test_select_dm_clears_unread() {
        let mut state = AppState::new();
        *state.unread_dms.entry(5).or_insert(0) = 2;
        state.select_dm(5);
        assert_eq!(state.active_conversation, Some(ConversationKind::DM(5)));
        assert_eq!(state.unread_dms.get(&5).copied().unwrap_or(0), 0);
    }

    #[test]
    fn test_add_channel_message_increments_unread_when_inactive() {
        let mut state = AppState::new();
        state.select_channel(1);
        let msg = make_message(1, 2, "hi"); // channel 2, not active
        state.add_channel_message(msg);
        assert_eq!(state.unread_channels.get(&2).copied().unwrap_or(0), 1);
    }

    #[test]
    fn test_add_channel_message_no_unread_when_active() {
        let mut state = AppState::new();
        state.select_channel(1);
        let msg = make_message(1, 1, "hi");
        state.add_channel_message(msg);
        assert_eq!(state.unread_channels.get(&1).copied().unwrap_or(0), 0);
        assert_eq!(state.channel_messages[&1].len(), 1);
    }

    #[test]
    fn test_add_dm_message_increments_unread_when_inactive() {
        let mut state = AppState::new();
        state.current_user = Some(make_user(1, "alice"));
        state.select_channel(10); // active on a channel, not the DM
        let dm = make_dm(1, 2, 1, "hey"); // sender=2, recipient=1(me)
        state.add_dm_message(dm, 1);
        assert_eq!(state.unread_dms.get(&2).copied().unwrap_or(0), 1);
    }

    #[test]
    fn test_input_buffer_operations() {
        let mut state = AppState::new();
        state.push_input_char('h');
        state.push_input_char('i');
        assert_eq!(state.input_buffer, "hi");

        state.pop_input_char();
        assert_eq!(state.input_buffer, "h");

        let taken = state.take_input();
        assert_eq!(taken, "h");
        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn test_sidebar_items_excludes_self() {
        let mut state = AppState::new();
        state.current_user = Some(make_user(1, "alice"));
        state.users = vec![make_user(1, "alice"), make_user(2, "bob"), make_user(3, "charlie")];
        state.channels = vec![Channel {
            id: 1,
            name: "general".to_string(),
            description: "".to_string(),
            owner_id: 1,
            created_at: "2026-01-01".to_string(),
        }];

        let items = state.sidebar_items();
        // 1 channel + 2 users (bob + charlie, not alice)
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_status_message() {
        let mut state = AppState::new();
        state.set_status("connecting...");
        assert_eq!(state.status_message.as_deref(), Some("connecting..."));
        state.clear_status();
        assert!(state.status_message.is_none());
    }
}
