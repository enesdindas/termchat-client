#[cfg(test)]
mod tests {
    use crate::models::*;

    #[test]
    fn test_ws_envelope_serialize() {
        let env = WsEnvelope::new(
            "message.send",
            serde_json::json!({"channel_id": 1, "content": "hello"}),
        );
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"type\":\"message.send\""));
        assert!(json.contains("\"channel_id\":1"));
    }

    #[test]
    fn test_ws_envelope_deserialize() {
        let json = r#"{"type":"message.new","payload":{"id":1,"channel_id":2,"author_id":3,"author_username":"alice","content":"hi","created_at":"2026-01-01T00:00:00Z"}}"#;
        let env: WsEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(env.event_type, "message.new");
        let msg: Message = serde_json::from_value(env.payload).unwrap();
        assert_eq!(msg.content, "hi");
        assert_eq!(msg.author_username, "alice");
    }

    #[test]
    fn test_message_roundtrip() {
        let json = r#"{"id":5,"channel_id":2,"author_id":1,"author_username":"bob","content":"hello world","created_at":"2026-04-07T12:00:00Z"}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.id, 5);
        assert_eq!(msg.content, "hello world");
        let out = serde_json::to_string(&msg).unwrap();
        assert!(out.contains("\"id\":5"));
    }

    #[test]
    fn test_dm_roundtrip() {
        let json = r#"{"id":1,"sender_id":2,"sender_username":"alice","recipient_id":3,"content":"hey","created_at":"2026-04-07T12:00:00Z"}"#;
        let dm: DirectMessage = serde_json::from_str(json).unwrap();
        assert_eq!(dm.sender_username, "alice");
        assert_eq!(dm.content, "hey");
    }

    #[test]
    fn test_user_roundtrip() {
        let json = r#"{"id":1,"username":"alice","created_at":"2026-04-07T00:00:00Z"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.username, "alice");
    }

    #[test]
    fn test_channel_roundtrip() {
        let json = r#"{"id":1,"name":"general","description":"main channel","owner_id":1,"created_at":"2026-04-07T00:00:00Z"}"#;
        let ch: Channel = serde_json::from_str(json).unwrap();
        assert_eq!(ch.name, "general");
        assert_eq!(ch.owner_id, 1);
    }

    #[test]
    fn test_login_request_serialize() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "secret".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"password\":\"secret\""));
    }
}
