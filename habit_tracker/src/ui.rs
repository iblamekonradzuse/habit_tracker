use crate::habit::Habit;
use chrono::NaiveDate;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(8),
        ])
        .split(f.size());

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let title = Paragraph::new(Span::raw(format!("(Date: {})", current_date)))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            "Habit Tracker",
            Style::default().add_modifier(Modifier::BOLD),
        )));
    f.render_widget(title, header_chunks[0]);

    let input = Paragraph::new(app_state.new_habit_name.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Add New Habit"),
        );
    f.render_widget(input, header_chunks[1]);

    if let InputMode::AddingHabit = app_state.input_mode {
        f.set_cursor(
            header_chunks[1].x + app_state.new_habit_name.len() as u16 + 1,
            header_chunks[1].y + 1,
        );
    }

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled("Habits", Style::default().fg(Color::Magenta))),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        );

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    f.render_stateful_widget(habits_list, main_chunks[0], &mut ListState::default());

    let streak_chart = create_streak_chart(habits, *current_date);
    f.render_widget(streak_chart, main_chunks[1]);

    draw_help(f, chunks[2]);
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled("Streaks", Style::default().fg(Color::Blue))),
        )
        .style(Style::default())
}

fn draw_help<B: Backend>(f: &mut Frame<B>, area: tui::layout::Rect) {
    let help_text = vec![
        Spans::from(Span::styled(
            "q: Quit",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "a: Add new habit",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "Enter: Mark/unmark selected habit",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "d: Delete selected habit",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "Left/Right Arrow: Change date",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "Up/Down Arrow: Navigate habits",
            Style::default().add_modifier(Modifier::BOLD),
        )),
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

