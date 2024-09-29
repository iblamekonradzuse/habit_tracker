use crate::habit::{Frequency, Habit};
use crate::todo::Todo;
use chrono::{Datelike, NaiveDate};
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
    AddingTodo,
    EditingCategory,
    EditingHabit,
}

pub struct AppState {
    pub selected: Option<usize>,
    pub input_mode: InputMode,
    pub new_habit_name: String,
    pub new_category: String,
    pub new_habit_frequency: Frequency,
    pub new_todo: String,
    pub current_tab: usize,
    pub total_items: usize,
    pub list_items: Vec<ListEntry>,
    pub current_week: NaiveDate,
    pub edit_buffer: String,
}

pub enum ListEntry {
    Category(String),
    Habit(Habit),
    Todo(Todo),
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            selected: None,
            input_mode: InputMode::Normal,
            new_habit_name: String::new(),
            new_category: String::new(),
            new_habit_frequency: Frequency::Daily,
            new_todo: String::new(),
            current_tab: 0,
            total_items: 0,
            list_items: Vec::new(),
            current_week: chrono::Local::now().date_naive(),
            edit_buffer: String::new(),
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

    pub fn update_list_items(&mut self, habits: &[Habit], todos: &[Todo]) {
        self.list_items.clear();
        let filtered_habits: Vec<&Habit> = habits
            .iter()
            .filter(|h| match self.current_tab {
                0 => h.get_frequency() == Frequency::Daily,
                1 => h.get_frequency() == Frequency::Weekly,
                2 => h.get_frequency() == Frequency::Monthly,
                3 => true, // Show all habits in the Frequency tab
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

        if self.current_tab == 4 {
            // Todo tab
            self.list_items
                .push(ListEntry::Category("To-Do List".to_string()));
            for todo in todos {
                self.list_items.push(ListEntry::Todo(todo.clone()));
            }
        }

        self.total_items = self.list_items.len();
    }

    pub fn next_week(&mut self) {
        self.current_week = self.current_week + chrono::Duration::days(7);
    }

    pub fn previous_week(&mut self) {
        self.current_week = self.current_week - chrono::Duration::days(7);
    }
}

pub fn draw<B: Backend>(
    f: &mut Frame<B>,
    habits: &[Habit],
    todos: &[Todo],
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
    draw_main_content(f, chunks[2], habits, todos, current_date, app_state);
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
    let titles = vec!["Daily", "Weekly", "Monthly", "Frequency", "Todo"];
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
    _todos: &[Todo],
    current_date: &NaiveDate,
    app_state: &mut AppState,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[0]);

    draw_input(f, input_chunks[0], app_state);
    draw_list(f, input_chunks[1], current_date, app_state);

    if app_state.current_tab == 3 {
        // Frequency tab
        draw_frequency_graph(f, chunks[1], habits, app_state);
    } else {
        draw_streak_chart(f, chunks[1], habits, current_date);
    }
}

fn draw_list<B: Backend>(
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
            ListEntry::Todo(todo) => {
                let icon = if todo.completed { "✅" } else { "⬜" };
                let content = Spans::from(vec![
                    Span::raw("  "), // Indent todo
                    Span::raw(format!("{} ", icon)),
                    Span::styled(&todo.description, Style::default().fg(Color::White)),
                ]);
                items.push(ListItem::new(content));
            }
        }
    }

    let list_title = match app_state.current_tab {
        0 => "Daily Habits",
        1 => "Weekly Habits",
        2 => "Monthly Habits",
        3 => "All Habits",
        4 => "Todo List",
        _ => "Items",
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(list_title, Style::default().fg(Color::Cyan))),
        )
        .highlight_style(
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(app_state.selected);
    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_input<B: Backend>(f: &mut Frame<B>, area: Rect, app_state: &AppState) {
    let (input_text, input_prompt) = match app_state.input_mode {
        InputMode::Normal => ("", ""),
        InputMode::AddingCategory => (app_state.new_category.as_str(), "Enter category: "),
        InputMode::AddingHabit => (app_state.new_habit_name.as_str(), "Enter habit name: "),
        InputMode::AddingTodo => (app_state.new_todo.as_str(), "Enter todo: "),
        InputMode::EditingCategory => (app_state.edit_buffer.as_str(), "Edit category: "),
        InputMode::EditingHabit => (app_state.edit_buffer.as_str(), "Edit habit name: "),
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

fn draw_frequency_graph<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    habits: &[Habit],
    app_state: &AppState,
) {
    let mut content = Vec::new();

    // Calculate the start and end of the week
    let week_start = app_state.current_week
        - chrono::Duration::days(app_state.current_week.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(6);

    // Add centered week navigation, shifted one space to the left
    let week_nav = format!(
        "'p' Prev Week    {0} - {1}    Next Week 'n'",
        week_start.format("%m-%d"),
        week_end.format("%m-%d")
    );
    let left_padding = (area.width as usize - week_nav.len()) / 2 - 1; // Subtract 1 to shift left
    content.push(Spans::from(vec![
        Span::raw(" ".repeat(left_padding)),
        Span::styled(week_nav, Style::default().fg(Color::Yellow)),
    ]));
    content.push(Spans::from(vec![Span::raw("")])); // Empty line

    // Add weekday headers, shifted one space to the right
    let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let habit_name_width = 15; // Increased by 1 to shift calendar right
    let weekday_spans: Vec<Span> = vec![Span::raw(" ".repeat(habit_name_width - 1))]
        .into_iter()
        .chain(
            weekdays
                .iter()
                .map(|&day| Span::styled(format!("{:^5}", day), Style::default().fg(Color::Cyan))),
        )
        .collect();
    content.push(Spans::from(weekday_spans));

    // Add horizontal line, extended by one character
    content.push(Spans::from(vec![Span::raw(
        "─".repeat(habit_name_width + weekdays.len() * 5),
    )]));

    for habit in habits {
        let mut habit_line = Vec::new();
        habit_line.push(Span::styled(
            format!("{:<width$}", habit.name, width = habit_name_width),
            Style::default().fg(Color::Yellow),
        ));

        for day_offset in 0..7 {
            let date = week_start + chrono::Duration::days(day_offset);
            let symbol = if habit.is_completed(date) {
                Span::styled("[X]", Style::default().fg(Color::Green))
            } else {
                Span::styled("[ ]", Style::default().fg(Color::Red))
            };
            habit_line.push(symbol);
            habit_line.push(Span::raw("  ")); // Add space between boxes
        }

        content.push(Spans::from(habit_line));
    }

    let frequency_graph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Frequency Graph"),
        )
        .alignment(tui::layout::Alignment::Left)
        .wrap(tui::widgets::Wrap { trim: false });

    f.render_widget(frequency_graph, area);
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
        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Edit | "),
        Span::styled("←/→", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Date | "),
        Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Nav | "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Switch tab  "),
    ])];

    let help_paragraph = Paragraph::new(help_text)
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help_paragraph, area);
}
