use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::state::{AppState, ConversationKind, SidebarItem};

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let items: Vec<ListItem> = state
        .sidebar_items()
        .iter()
        .map(|item| match item {
            SidebarItem::Channel(ch) => {
                let unread = state.unread_channels.get(&ch.id).copied().unwrap_or(0);
                let is_active =
                    state.active_conversation == Some(ConversationKind::Channel(ch.id));

                let style = if is_active {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let label = if unread > 0 {
                    format!(" # {}  ({})", ch.name, unread)
                } else {
                    format!(" # {}", ch.name)
                };

                ListItem::new(Line::from(vec![Span::styled(label, style)]))
            }
            SidebarItem::User(user) => {
                let partner_id = user.id;
                let unread = state.unread_dms.get(&partner_id).copied().unwrap_or(0);
                let is_active =
                    state.active_conversation == Some(ConversationKind::DM(partner_id));

                let style = if is_active {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::LightMagenta)
                };

                let label = if unread > 0 {
                    format!(" @ {}  ({})", user.username, unread)
                } else {
                    format!(" @ {}", user.username)
                };

                ListItem::new(Line::from(vec![Span::styled(label, style)]))
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Channels & DMs ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(list, chunks[0]);
}
