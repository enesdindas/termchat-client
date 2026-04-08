use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let title = if let Some(status) = &state.status_message {
        format!(" {} ", status)
    } else {
        let ws = match state.ws_lifecycle {
            crate::state::WsLifecycle::Connected => "online",
            crate::state::WsLifecycle::Connecting => "connecting",
            crate::state::WsLifecycle::Reconnecting => "reconnecting",
            crate::state::WsLifecycle::Disconnected => "offline",
        };
        format!(" Message ({} | Enter to send) ", ws)
    };

    // Show cursor at end
    let content = Line::from(vec![
        Span::raw(state.input_buffer.as_str()),
        Span::styled("█", Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK)),
    ]);

    let para = Paragraph::new(content).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(para, area);
}
