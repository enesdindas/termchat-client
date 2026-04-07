use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::state::{AppState, LoginField};

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    // Center a box
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(14),
            Constraint::Percentage(30),
        ])
        .split(area);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Min(40),
            Constraint::Percentage(25),
        ])
        .split(vert[1]);

    let panel_area = horiz[1];
    f.render_widget(Clear, panel_area);

    let block = Block::default()
        .title(" termchat — Login ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, panel_area);

    let inner = inner_area(panel_area, 1);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(3), // username
            Constraint::Length(1), // spacer
            Constraint::Length(3), // password
            Constraint::Length(1), // spacer
            Constraint::Length(1), // hint
            Constraint::Length(1), // error/status
        ])
        .split(inner);

    // Username field
    let username_style = if state.login_field == LoginField::Username {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let username_block = Block::default()
        .title("Username")
        .borders(Borders::ALL)
        .border_style(username_style);
    let username_text = Paragraph::new(state.login_username.as_str())
        .block(username_block)
        .style(Style::default().fg(Color::White));
    f.render_widget(username_text, chunks[1]);

    // Password field
    let pass_style = if state.login_field == LoginField::Password {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let pass_block = Block::default()
        .title("Password")
        .borders(Borders::ALL)
        .border_style(pass_style);
    let masked: String = "*".repeat(state.login_password.len());
    let pass_text = Paragraph::new(masked.as_str())
        .block(pass_block)
        .style(Style::default().fg(Color::White));
    f.render_widget(pass_text, chunks[3]);

    // Hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" switch field  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" login  "),
        Span::styled("Ctrl+R", Style::default().fg(Color::Cyan)),
        Span::raw(" register"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[5]);

    // Error or status
    if let Some(err) = &state.login_error {
        let error_text = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_text, chunks[6]);
    } else if let Some(status) = &state.login_status {
        let status_text = Paragraph::new(status.as_str())
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(status_text, chunks[6]);
    }
}

fn inner_area(area: Rect, margin: u16) -> Rect {
    Rect {
        x: area.x + margin + 1,
        y: area.y + margin + 1,
        width: area.width.saturating_sub((margin + 1) * 2),
        height: area.height.saturating_sub((margin + 1) * 2),
    }
}
