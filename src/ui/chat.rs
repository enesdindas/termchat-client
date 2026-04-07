use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::{AppState, ConversationKind};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let title = match &state.active_conversation {
        Some(ConversationKind::Channel(id)) => {
            if let Some(ch) = state.channels.iter().find(|c| c.id == *id) {
                format!(" # {} ", ch.name)
            } else {
                " Chat ".to_string()
            }
        }
        Some(ConversationKind::DM(partner_id)) => {
            if let Some(u) = state.users.iter().find(|u| u.id == *partner_id) {
                format!(" @ {} ", u.username)
            } else {
                " DM ".to_string()
            }
        }
        None => " Select a channel or DM ".to_string(),
    };

    let lines: Vec<Line> = build_lines(state);

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((state.chat_scroll, 0));

    f.render_widget(para, area);
}

fn build_lines(state: &AppState) -> Vec<Line<'static>> {
    match &state.active_conversation {
        Some(ConversationKind::Channel(channel_id)) => {
            let msgs = state.channel_messages.get(channel_id);
            match msgs {
                None => vec![Line::from(Span::styled(
                    "No messages yet. Start typing!",
                    Style::default().fg(Color::DarkGray),
                ))],
                Some(q) if q.is_empty() => vec![Line::from(Span::styled(
                    "No messages yet. Start typing!",
                    Style::default().fg(Color::DarkGray),
                ))],
                Some(q) => q
                    .iter()
                    .map(|msg| {
                        let time = format_time(&msg.created_at);
                        Line::from(vec![
                            Span::styled(
                                format!("[{}] ", time),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                format!("{}: ", msg.author_username),
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(msg.content.clone()),
                        ])
                    })
                    .collect(),
            }
        }
        Some(ConversationKind::DM(partner_id)) => {
            let dms = state.dm_messages.get(partner_id);
            let my_id = state.current_user.as_ref().map(|u| u.id).unwrap_or(0);
            match dms {
                None => vec![Line::from(Span::styled(
                    "No messages yet.",
                    Style::default().fg(Color::DarkGray),
                ))],
                Some(q) if q.is_empty() => vec![Line::from(Span::styled(
                    "No messages yet.",
                    Style::default().fg(Color::DarkGray),
                ))],
                Some(q) => q
                    .iter()
                    .map(|dm| {
                        let time = format_time(&dm.created_at);
                        let (name, color) = if dm.sender_id == my_id {
                            ("you".to_string(), Color::Green)
                        } else {
                            (dm.sender_username.clone(), Color::Magenta)
                        };
                        Line::from(vec![
                            Span::styled(
                                format!("[{}] ", time),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(
                                format!("{}: ", name),
                                Style::default().fg(color).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(dm.content.clone()),
                        ])
                    })
                    .collect(),
            }
        }
        None => vec![Line::from(Span::styled(
            "Select a channel or DM from the sidebar",
            Style::default().fg(Color::DarkGray),
        ))],
    }
}

fn format_time(ts: &str) -> String {
    // ts is like "2026-04-07T12:34:56Z" or "2026-04-07 12:34:56"
    if let Some(t) = ts.get(11..16) {
        t.to_string()
    } else {
        ts.to_string()
    }
}
