use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod habit;
mod storage;
mod todo;
mod ui;

use crate::ui::ListEntry;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up application state
    let mut habits = storage::load_habits()?;
    let mut todos = storage::load_todos()?;
    let mut current_date = chrono::Local::now().date_naive();
    let mut app_state = ui::AppState::default();

    // Run the main application loop
    let res = run_app(
        &mut terminal,
        &mut habits,
        &mut todos,
        &mut current_date,
        &mut app_state,
    );

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err)
    }

    // Save habits and todos before exiting
    storage::save_habits(&habits)?;
    storage::save_todos(&todos)?;

    Ok(())
}

fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    habits: &mut Vec<habit::Habit>,
    todos: &mut Vec<todo::Todo>,
    current_date: &mut chrono::NaiveDate,
    app_state: &mut ui::AppState,
) -> io::Result<()> {
    app_state.update_list_items(habits, todos);
    loop {
        terminal.draw(|f| ui::draw(f, habits, todos, current_date, app_state))?;

        if let Event::Key(key) = event::read()? {
            match app_state.input_mode {
                ui::InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => {
                        if app_state.current_tab == 4 {
                            app_state.input_mode = ui::InputMode::AddingTodo;
                            app_state.new_todo.clear();
                        } else {
                            app_state.input_mode = ui::InputMode::AddingCategory;
                            app_state.new_category.clear();
                            app_state.new_habit_name.clear();
                            app_state.new_habit_frequency = habit::Frequency::Daily;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(index) = app_state.selected {
                            match &app_state.list_items[index] {
                                ListEntry::Category(category) => {
                                    // Toggle all habits in the category
                                    let all_completed = habits
                                        .iter()
                                        .filter(|h| h.category == *category)
                                        .all(|h| h.is_completed(*current_date));
                                    for habit in habits.iter_mut().filter(|h| h.category == *category) {
                                        if all_completed {
                                            habit.unmark_completed(*current_date);
                                        } else {
                                            habit.mark_completed(*current_date);
                                        }
                                    }
                                }
                                ListEntry::Habit(selected_habit) => {
                                    if let Some(habit) = habits.iter_mut().find(|h| h.name == selected_habit.name && h.category == selected_habit.category) {
                                        if habit.is_completed(*current_date) {
                                            habit.unmark_completed(*current_date);
                                        } else {
                                            habit.mark_completed(*current_date);
                                        }
                                    }
                                }
                                ListEntry::Todo(selected_todo) => {
                                    if let Some(todo) = todos.iter_mut().find(|t| t.description == selected_todo.description) {
                                        todo.toggle_completion();
                                    }
                                }
                            }
                            app_state.update_list_items(habits, todos);
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(index) = app_state.selected {
                            match &app_state.list_items[index] {
                                ListEntry::Category(category) => {
                                    // Remove all habits in the category
                                    habits.retain(|h| h.category != *category);
                                }
                                ListEntry::Habit(selected_habit) => {
                                    // Remove the selected habit
                                    habits.retain(|h| h.name != selected_habit.name || h.category != selected_habit.category);
                                }
                                ListEntry::Todo(selected_todo) => {
                                    // Remove the selected todo
                                    todos.retain(|t| t.description != selected_todo.description);
                                }
                            }
                            app_state.update_list_items(habits, todos);
                            if !app_state.list_items.is_empty() {
                                app_state.selected = Some(index.min(app_state.total_items - 1));
                            } else {
                                app_state.selected = None;
                            }
                        }
                    }
                    KeyCode::Left => {
                        *current_date = current_date.pred_opt().unwrap_or(*current_date)
                    }
                    KeyCode::Right => {
                        *current_date = current_date.succ_opt().unwrap_or(*current_date)
                    }
                    KeyCode::Up => {
                        app_state.previous();
                    }
                    KeyCode::Down => {
                        app_state.next();
                    }
                    KeyCode::Tab => {
                        app_state.current_tab = (app_state.current_tab + 1) % 5;
                        app_state.selected = None;
                        app_state.update_list_items(habits, todos);
                    }
                        KeyCode::Char('p') => {
                        app_state.previous_week();
                    },
                    KeyCode::Char('n') => {
                        app_state.next_week();
                    },
                    _ => {}
                },
                ui::InputMode::AddingCategory => match key.code {
                    KeyCode::Enter => {
                        app_state.input_mode = ui::InputMode::AddingHabit;
                    }
                    KeyCode::Esc => {
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_category.clear();
                        app_state.new_habit_name.clear();
                    }
                    KeyCode::Char(c) => {
                        app_state.new_category.push(c);
                    }
                    KeyCode::Backspace => {
                        app_state.new_category.pop();
                    }
                    _ => {}
                },
                ui::InputMode::AddingHabit => match key.code {
                    KeyCode::Enter => {
                        let new_habit = habit::Habit::new(
                            app_state.new_habit_name.clone(),
                            app_state.new_category.clone(),
                            app_state.new_habit_frequency,
                        );
                        habits.push(new_habit);
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_habit_name.clear();
                        app_state.new_category.clear();
                        app_state.new_habit_frequency = habit::Frequency::Daily;
                        app_state.update_list_items(habits, todos);
                    }
                    KeyCode::Esc => {
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_habit_name.clear();
                        app_state.new_category.clear();
                    }
                    KeyCode::Char(c) => {
                        app_state.new_habit_name.push(c);
                    }
                    KeyCode::Backspace => {
                        app_state.new_habit_name.pop();
                    }
                    KeyCode::Tab => {
                        app_state.new_habit_frequency = match app_state.new_habit_frequency {
                            habit::Frequency::Daily => habit::Frequency::Weekly,
                            habit::Frequency::Weekly => habit::Frequency::Monthly,
                            habit::Frequency::Monthly => habit::Frequency::Daily,
                        };
                    }
                    _ => {}
                },
                ui::InputMode::AddingTodo => match key.code {
                    KeyCode::Enter => {
                        let new_todo = todo::Todo::new(app_state.new_todo.clone());
                        todos.push(new_todo);
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_todo.clear();
                        app_state.update_list_items(habits, todos);
                    }
                    KeyCode::Esc => {
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_todo.clear();
                    }
                    KeyCode::Char(c) => {
                        app_state.new_todo.push(c);
                    }
                    KeyCode::Backspace => {
                        app_state.new_todo.pop();
                    }
                    _ => {}
                },
            }
        }
    }
}
