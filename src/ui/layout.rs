use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::state::{AppState, Screen};
use super::{chat, input, login, sidebar};

pub fn render(f: &mut Frame, state: &AppState) {
    match state.screen {
        Screen::Login => {
            login::render(f, state);
        }
        Screen::Main => {
            render_main(f, state);
        }
    }
}

fn render_main(f: &mut Frame, state: &AppState) {
    let area = f.area();

    // Horizontal split: sidebar (22%) | right (78%)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(22), Constraint::Percentage(78)])
        .split(area);

    // Right side vertical split: chat (90%) | input (10%, min 3)
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(columns[1]);

    sidebar::render(f, columns[0], state);
    chat::render(f, right_rows[0], state);
    input::render(f, right_rows[1], state);
}
