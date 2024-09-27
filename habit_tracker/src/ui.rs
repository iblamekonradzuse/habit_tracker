use crate::habit::{Frequency, Habit};
use chrono::NaiveDate;
use std::collections::BTreeMap;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

pub enum InputMode {
    Normal,
    AddingCategory,
    AddingHabit,
}

pub struct AppState {
    pub selected: Option<usize>,
    pub input_mode: InputMode,
    pub new_habit_name: String,
    pub new_category: String,
    pub new_habit_frequency: Frequency,
    pub current_tab: usize,
    pub total_items: usize,
    pub list_items: Vec<ListEntry>,
}

pub enum ListEntry {
    Category(String),
    Habit(Habit),
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            selected: None,
            input_mode: InputMode::Normal,
            new_habit_name: String::new(),
            new_category: String::new(),
            new_habit_frequency: Frequency::Daily,
            current_tab: 0,
            total_items: 0,
            list_items: Vec::new(),
        }
    }
}

impl AppState {
    pub fn next(&mut self) {
        self.selected = Some(match self.selected {
            Some(i) => (i + 1) % self.total_items,
            None => 0,
        });
    }

    pub fn previous(&mut self) {
        self.selected = Some(match self.selected {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    self.total_items - 1
                }
            }
            None => 0,
        });
    }

    pub fn update_list_items(&mut self, habits: &[Habit]) {
        self.list_items.clear();
        let filtered_habits: Vec<&Habit> = habits
            .iter()
            .filter(|h| match self.current_tab {
                0 => h.get_frequency() == Frequency::Daily,
                1 => h.get_frequency() == Frequency::Weekly,
                2 => h.get_frequency() == Frequency::Monthly,
                _ => false,
            })
            .collect();

        let mut grouped_habits: BTreeMap<&str, Vec<&Habit>> = BTreeMap::new();
        for habit in &filtered_habits {
            grouped_habits
                .entry(&habit.category)
                .or_insert_with(Vec::new)
                .push(habit);
        }

        for (category, habits) in grouped_habits {
            self.list_items
                .push(ListEntry::Category(category.to_string()));
            for habit in habits {
                self.list_items.push(ListEntry::Habit((*habit).clone()));
            }
        }

        self.total_items = self.list_items.len();
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
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    draw_title(f, chunks[0], current_date);
    draw_tabs(f, chunks[1], app_state);
    draw_main_content(f, chunks[2], habits, current_date, app_state);
    draw_help(f, chunks[3]);
}

fn draw_title<B: Backend>(f: &mut Frame<B>, area: Rect, current_date: &NaiveDate) {
    let title = Paragraph::new(Span::styled(
        format!("Habit Tracker - {}", current_date),
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .alignment(tui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_tabs<B: Backend>(f: &mut Frame<B>, area: Rect, app_state: &AppState) {
    let titles = vec!["Daily", "Weekly", "Monthly"];
    let tabs = Tabs::new(titles.into_iter().map(Spans::from).collect())
        .select(app_state.current_tab)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw("|"));
    f.render_widget(tabs, area);
}

fn draw_main_content<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    habits: &[Habit],
    current_date: &NaiveDate,
    app_state: &mut AppState,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[0]);

    draw_input(f, input_chunks[0], app_state);
    draw_habits_list(f, input_chunks[1], current_date, app_state);
    draw_streak_chart(f, chunks[1], habits, current_date);
}

fn draw_habits_list<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    current_date: &NaiveDate,
    app_state: &mut AppState,
) {
    let mut items = Vec::new();
    let category_colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];
    let mut color_index = 0;

    for (_index, entry) in app_state.list_items.iter().enumerate() {
        match entry {
            ListEntry::Category(category) => {
                let category_color = category_colors[color_index % category_colors.len()];
                color_index += 1;
                items.push(ListItem::new(Spans::from(vec![Span::styled(
                    format!("{}:", category),
                    Style::default()
                        .fg(category_color)
                        .add_modifier(Modifier::BOLD),
                )])));
            }
            ListEntry::Habit(habit) => {
                let completed = habit.is_completed(*current_date);
                let icon = if completed { "✅" } else { "⬜" };
                let content = Spans::from(vec![
                    Span::raw("  "), // Indent habit
                    Span::raw(format!("{} ", icon)),
                    Span::styled(
                        &habit.name,
                        Style::default()
                            .fg(category_colors[(color_index - 1) % category_colors.len()]),
                    ),
                ]);
                items.push(ListItem::new(content));
            }
        }
    }

    let habits_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Habits", Style::default().fg(Color::Cyan))),
        )
        .highlight_style(
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(app_state.selected);
    f.render_stateful_widget(habits_list, area, &mut list_state);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, area: Rect, app_state: &AppState) {
    let input_text = match app_state.input_mode {
        InputMode::Normal => "",
        InputMode::AddingCategory => &app_state.new_category,
        InputMode::AddingHabit => &app_state.new_habit_name,
    };

    let input_prompt = match app_state.input_mode {
        InputMode::Normal => "",
        InputMode::AddingCategory => "Enter category: ",
        InputMode::AddingHabit => "Enter habit name: ",
    };

    let frequency_text = match app_state.input_mode {
        InputMode::AddingHabit => format!(" ({})", app_state.new_habit_frequency),
        _ => String::new(),
    };

    let input = Paragraph::new(format!("{}{}{}", input_prompt, input_text, frequency_text))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, area);
}

fn draw_streak_chart<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    habits: &[Habit],
    current_date: &NaiveDate,
) {
    let mut content = Vec::new();

    for habit in habits {
        let streak = habit.get_streak(*current_date);
        let bar = "█".repeat(streak.min(20) as usize);
        content.push(Spans::from(vec![
            Span::styled(&habit.name, Style::default().fg(Color::Yellow)),
            Span::raw(": "),
            Span::styled(bar, Style::default().fg(Color::Green)),
            Span::raw(format!(" {}", streak)),
        ]));
    }

    let streak_chart = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Streaks"))
        .wrap(tui::widgets::Wrap { trim: true });

    f.render_widget(streak_chart, area);
}

fn draw_help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let help_text = vec![Spans::from(vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Quit | "),
        Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Add | "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Toggle | "),
        Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Delete | "),
        Span::styled("←/→", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Change date | "),
        Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Navigate | "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Switch frequency"),
    ])];

    let help_paragraph = Paragraph::new(help_text)
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help_paragraph, area);
}

