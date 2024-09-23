use crate::habit::Habit;
use chrono::NaiveDate;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub enum InputMode {
    Normal,
    AddingHabit,
}

pub struct AppState {
    pub selected: Option<usize>,
    pub input_mode: InputMode,
    pub new_habit_name: String,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            selected: None,
            input_mode: InputMode::Normal,
            new_habit_name: String::new(),
        }
    }
}

impl AppState {
    pub fn next(&mut self, len: usize) {
        self.selected = match self.selected {
            Some(i) => Some((i + 1) % len),
            None => Some(0),
        };
    }

    pub fn previous(&mut self) {
        self.selected = match self.selected {
            Some(i) => Some(if i == 0 { 0 } else { i - 1 }),
            None => None,
        };
    }
}

pub fn draw<B: Backend>(
    f: &mut Frame<B>,
    habits: &[Habit],
    current_date: &NaiveDate,
    app_state: &mut AppState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Percentage(70), Constraint::Length(8)])
        .split(f.size());

    let title = Paragraph::new(Spans::from(vec![
        Span::styled(
            "Habit Tracker ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("(Date: {})", current_date)),
    ]))
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let habits_list: Vec<ListItem> = habits
        .iter()
        .enumerate()
        .map(|(i, habit)| {
            let completed = habit.is_completed(*current_date);
            let icon = if completed { "✅" } else { "⬜" };
            let content = vec![Spans::from(Span::raw(format!("{} {}", icon, habit.name)))];
            let style = if Some(i) == app_state.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let habits_list = List::new(habits_list)
        .block(Block::default().borders(Borders::ALL).title("Habits"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        );

    let streak_chart = create_streak_chart(habits, *current_date);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    f.render_stateful_widget(habits_list, bottom_chunks[0], &mut ListState::default());
    f.render_widget(streak_chart, bottom_chunks[1]);

    draw_help(f, chunks[2]);

    if let InputMode::AddingHabit = app_state.input_mode {
        draw_add_habit_popup(f, app_state);
    }
}

fn create_streak_chart<'a>(habits: &[Habit], end_date: NaiveDate) -> Paragraph<'a> {
    let mut content = Vec::new();

    for habit in habits {
        let streak = habit.get_streak(end_date);
        let bar = "█".repeat(streak as usize);
        content.push(Spans::from(vec![
            Span::raw(format!("{:<20}", habit.name)),
            Span::styled(bar, Style::default().fg(Color::Green)),
            Span::raw(format!(" {}", streak)),
        ]));
    }

    Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default())
}

fn draw_help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = vec![
        Spans::from(Span::styled("q: Quit", Style::default().add_modifier(Modifier::BOLD))),
        Spans::from(Span::styled("a: Add new habit", Style::default().add_modifier(Modifier::BOLD))),
        Spans::from(Span::styled("Enter: Mark/unmark selected habit", Style::default().add_modifier(Modifier::BOLD))),
        Spans::from(Span::styled("d: Delete selected habit", Style::default().add_modifier(Modifier::BOLD))),
        Spans::from(Span::styled("Left/Right Arrow: Change date", Style::default().add_modifier(Modifier::BOLD))),
        Spans::from(Span::styled("Up/Down Arrow: Navigate habits", Style::default().add_modifier(Modifier::BOLD))),
    ];

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let help_paragraph_left = Paragraph::new(help_text[0..3].to_vec())
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    let help_paragraph_right = Paragraph::new(help_text[3..].to_vec())
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help_paragraph_left, help_chunks[0]);
    f.render_widget(help_paragraph_right, help_chunks[1]);
}

fn draw_add_habit_popup<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState) {
    let popup_area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, popup_area);

    let popup = Block::default()
        .title("Add New Habit")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    f.render_widget(popup, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(popup_area);

    let input = Paragraph::new(app_state.new_habit_name.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Habit Name"));

    f.render_widget(input, chunks[0]);

    // Set cursor position
    f.set_cursor(
        chunks[0].x + app_state.new_habit_name.len() as u16 + 1,
        chunks[0].y + 1,
    );

    let help_text = Paragraph::new("Press Enter to confirm, Esc to cancel")
        .style(Style::default().fg(Color::Green))
        .alignment(tui::layout::Alignment::Center);

    f.render_widget(help_text, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
