use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::ui::common::centered_rect;

pub struct Item<T> {
    pub label: String,
    pub key: Option<char>,
    pub value: T,
}

pub struct State<T> {
    pub title: String,
    pub items: Vec<Item<T>>,
    pub current_index: usize,
}

#[derive(Clone)]
pub enum Selection<T> {
    Pending,
    Cancelled,
    Selected(T),
}

impl<T> State<T> {
    pub fn new(title: String, items: Vec<Item<T>>) -> Option<Self> {
        if items.is_empty() {
            None
        } else {
            Some(Self {
                title,
                items,
                current_index: 0,
            })
        }
    }
}

pub fn render<T>(state: &State<T>, frame: &mut Frame) {
    let modal_area = centered_rect(50, 60, frame.area());
    let accent_color = Color::Green;

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(format!(" {} ", state.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent_color));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let lines: Vec<Line> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.current_index;
            let bg = if is_selected {
                Color::DarkGray
            } else {
                Color::Reset
            };
            let key_span = match item.key {
                Some(k) => Span::styled(
                    format!(" [{}] ", k),
                    Style::default().fg(accent_color).bg(bg),
                ),
                None => Span::styled("     ", Style::default().bg(bg)),
            };
            let label_span = Span::styled(
                format!("{:<30}", item.label),
                Style::default()
                    .fg(if is_selected {
                        Color::White
                    } else {
                        Color::DarkGray
                    })
                    .bg(bg)
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );
            let cursor = Span::styled(
                if is_selected { " ▶ " } else { "   " },
                Style::default().fg(accent_color).bg(bg),
            );
            Line::from(vec![cursor, key_span, label_span])
        })
        .collect();

    frame.render_widget(Paragraph::new(Text::from(lines)), inner);
}

pub fn handle_key<T>(state: &mut State<T>, key: KeyEvent) -> Selection<&T> {
    match (key.code, key.modifiers.contains(KeyModifiers::CONTROL)) {
        (KeyCode::Char('j'), false) | (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
            state.current_index = (state.current_index + 1).min(state.items.len() - 1);
            Selection::Pending
        }
        (KeyCode::Char('k'), false) | (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
            state.current_index = state.current_index.saturating_sub(1);
            Selection::Pending
        }
        (KeyCode::Enter, _) | (KeyCode::Char('j'), true) => {
            Selection::Selected(&state.items[state.current_index].value)
        }
        (KeyCode::Esc, _) => Selection::Cancelled,
        (KeyCode::Char(c), false) => {
            if let Some(item) = state.items.iter().find(|i| i.key == Some(c)) {
                Selection::Selected(&item.value)
            } else {
                Selection::Pending
            }
        }
        _ => Selection::Pending,
    }
}
