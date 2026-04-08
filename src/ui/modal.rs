use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::state::{AppState, CreateChannelField, Modal};

pub fn render(f: &mut Frame, state: &AppState) {
    if !state.modal.is_open() {
        return;
    }
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    match &state.modal {
        Modal::None => {}
        Modal::CreateChannel { name, description, is_private, field, error } => {
            render_create_channel(f, area, name, description, *is_private, field, error);
        }
        Modal::ChannelList { cursor } => {
            render_channel_list(f, area, state, *cursor);
        }
        Modal::MembersList { channel_id, members, loading } => {
            render_members_list(f, area, state, *channel_id, members, *loading);
        }
        Modal::AddMember { channel_id, username_input, error } => {
            render_add_member(f, area, state, *channel_id, username_input, error);
        }
        Modal::RemoveMember { channel_id, members, cursor, loading } => {
            render_remove_member(f, area, state, *channel_id, members, *cursor, *loading);
        }
        Modal::ConfirmLogout => {
            render_confirm_logout(f, area);
        }
    }
}

fn render_create_channel(
    f: &mut Frame,
    area: Rect,
    name: &str,
    description: &str,
    is_private: bool,
    field: &CreateChannelField,
    error: &Option<String>,
) {
    let block = Block::default()
        .title(" Create Channel ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = inner_rect(area, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(3), // name
            Constraint::Length(3), // description
            Constraint::Length(1), // privacy
            Constraint::Length(1), // spacer
            Constraint::Length(1), // hint
            Constraint::Length(1), // error
        ])
        .split(inner);

    let name_block = Block::default()
        .title("Name")
        .borders(Borders::ALL)
        .border_style(field_style(field == &CreateChannelField::Name));
    f.render_widget(Paragraph::new(name).block(name_block), chunks[1]);

    let desc_block = Block::default()
        .title("Description")
        .borders(Borders::ALL)
        .border_style(field_style(field == &CreateChannelField::Description));
    f.render_widget(Paragraph::new(description).block(desc_block), chunks[2]);

    let priv_marker = if is_private { "[x]" } else { "[ ]" };
    let priv_text = format!("{} Private (only owner can add members)", priv_marker);
    let priv_para = Paragraph::new(priv_text)
        .style(field_style(field == &CreateChannelField::Privacy));
    f.render_widget(priv_para, chunks[3]);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" switch  "),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::raw(" toggle  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" create  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" cancel"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[5]);

    if let Some(err) = error {
        let p = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(p, chunks[6]);
    }
}

fn render_channel_list(f: &mut Frame, area: Rect, state: &AppState, cursor: usize) {
    let block = Block::default()
        .title(" Channels ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = inner_rect(area, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let items: Vec<ListItem> = state
        .channels
        .iter()
        .map(|ch| {
            let tag = if ch.is_private { "[priv]" } else { "#" };
            ListItem::new(format!("{} {}  — {}", tag, ch.name, ch.description))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !state.channels.is_empty() {
        list_state.select(Some(cursor.min(state.channels.len() - 1)));
    }
    f.render_stateful_widget(list, chunks[0], &mut list_state);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
        Span::raw(" move  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" select  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" close"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[1]);
}

fn render_members_list(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    channel_id: i64,
    members: &[crate::models::ChannelMember],
    loading: bool,
) {
    let title = state
        .channels
        .iter()
        .find(|c| c.id == channel_id)
        .map(|c| format!(" Members of #{} ", c.name))
        .unwrap_or_else(|| " Members ".to_string());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = inner_rect(area, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if loading {
        let p = Paragraph::new("Loading...").alignment(Alignment::Center);
        f.render_widget(p, chunks[0]);
    } else if members.is_empty() {
        let p = Paragraph::new("(no members)").alignment(Alignment::Center);
        f.render_widget(p, chunks[0]);
    } else {
        let items: Vec<ListItem> = members
            .iter()
            .map(|m| ListItem::new(format!("@{}", m.username)))
            .collect();
        let list = List::new(items);
        f.render_widget(list, chunks[0]);
    }

    let hint = Paragraph::new(Span::styled("Esc to close", Style::default().fg(Color::Cyan)))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[1]);
}

fn render_add_member(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    channel_id: i64,
    username_input: &str,
    error: &Option<String>,
) {
    let title = state
        .channels
        .iter()
        .find(|c| c.id == channel_id)
        .map(|c| format!(" Add user to #{} ", c.name))
        .unwrap_or_else(|| " Add user ".to_string());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = inner_rect(area, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let input_block = Block::default()
        .title("Username")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(
        Paragraph::new(username_input).block(input_block),
        chunks[1],
    );

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" add  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" cancel"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[3]);

    if let Some(err) = error {
        let p = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(p, chunks[4]);
    }
}

fn render_remove_member(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    channel_id: i64,
    members: &[crate::models::ChannelMember],
    cursor: usize,
    loading: bool,
) {
    let title = state
        .channels
        .iter()
        .find(|c| c.id == channel_id)
        .map(|c| format!(" Remove user from #{} ", c.name))
        .unwrap_or_else(|| " Remove user ".to_string());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let inner = inner_rect(area, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    if loading {
        let p = Paragraph::new("Loading...").alignment(Alignment::Center);
        f.render_widget(p, chunks[0]);
    } else if members.is_empty() {
        let p = Paragraph::new("(no members)").alignment(Alignment::Center);
        f.render_widget(p, chunks[0]);
    } else {
        let items: Vec<ListItem> = members
            .iter()
            .map(|m| ListItem::new(format!("@{}", m.username)))
            .collect();
        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        let mut list_state = ListState::default();
        list_state.select(Some(cursor.min(members.len() - 1)));
        f.render_stateful_widget(list, chunks[0], &mut list_state);
    }

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Up/Down", Style::default().fg(Color::Cyan)),
        Span::raw(" move  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" remove  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" cancel"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[1]);
}

fn render_confirm_logout(f: &mut Frame, area: Rect) {
    // Smaller centered area
    let block = Block::default()
        .title(" Logout ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);

    let inner = inner_rect(area, 1);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let p = Paragraph::new("Log out and clear saved token?")
        .alignment(Alignment::Center);
    f.render_widget(p, chunks[1]);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Green)),
        Span::raw(" confirm  "),
        Span::styled("n / Esc", Style::default().fg(Color::Cyan)),
        Span::raw(" cancel"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hint, chunks[2]);
}

fn field_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

fn inner_rect(area: Rect, margin: u16) -> Rect {
    Rect {
        x: area.x + margin + 1,
        y: area.y + margin + 1,
        width: area.width.saturating_sub((margin + 1) * 2),
        height: area.height.saturating_sub((margin + 1) * 2),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
