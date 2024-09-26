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
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up application state
    let mut habits = storage::load_habits()?;
    let mut current_date = chrono::Local::now().date_naive();
    let mut app_state = ui::AppState::default();

    // Run the main application loop
    let res = run_app(&mut terminal, &mut habits, &mut current_date, &mut app_state);

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

    // Save habits before exiting
    storage::save_habits(&habits)?;
    
    Ok(())
}

fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    habits: &mut Vec<habit::Habit>,
    current_date: &mut chrono::NaiveDate,
    app_state: &mut ui::AppState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, habits, current_date, app_state))?;

        if let Event::Key(key) = event::read()? {
            match app_state.input_mode {
                ui::InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => {
                        app_state.input_mode = ui::InputMode::AddingCategory;
                        app_state.new_category.clear();
                        app_state.new_habit_name.clear();
                        app_state.new_habit_frequency = habit::Frequency::Daily;
                    }
                    KeyCode::Enter => {
                        if let Some(index) = app_state.selected {
                            let mut filtered_habits: Vec<&mut habit::Habit> = habits
                                .iter_mut()
                                .filter(|h| match app_state.current_tab {
                                    0 => h.get_frequency() == habit::Frequency::Daily,
                                    1 => h.get_frequency() == habit::Frequency::Weekly,
                                    2 => h.get_frequency() == habit::Frequency::Monthly,
                                    _ => false,
                                })
                                .collect();

                            if let Some(habit) = filtered_habits.get_mut(index) {
                                if habit.is_completed(*current_date) {
                                    habit.unmark_completed(*current_date);
                                } else {
                                    habit.mark_completed(*current_date);
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(index) = app_state.selected {
                            let filtered_habits: Vec<usize> = habits
                                .iter()
                                .enumerate()
                                .filter(|(_, h)| match app_state.current_tab {
                                    0 => h.get_frequency() == habit::Frequency::Daily,
                                    1 => h.get_frequency() == habit::Frequency::Weekly,
                                    2 => h.get_frequency() == habit::Frequency::Monthly,
                                    _ => false,
                                })
                                .map(|(i, _)| i)
                                .collect();

                            if let Some(&habit_index) = filtered_habits.get(index) {
                                habits.remove(habit_index);
                                if index >= filtered_habits.len() - 1 {
                                    app_state.selected = Some(filtered_habits.len() - 2);
                                }
                            }
                        }
                    }
                    KeyCode::Left => *current_date = current_date.pred_opt().unwrap_or(*current_date),
                    KeyCode::Right => *current_date = current_date.succ_opt().unwrap_or(*current_date),
                    KeyCode::Up => {
                        let filtered_habits: Vec<&habit::Habit> = habits
                            .iter()
                            .filter(|h| match app_state.current_tab {
                                0 => h.get_frequency() == habit::Frequency::Daily,
                                1 => h.get_frequency() == habit::Frequency::Weekly,
                                2 => h.get_frequency() == habit::Frequency::Monthly,
                                _ => false,
                            })
                            .collect();
                        app_state.previous();
                    }
                    KeyCode::Down => {
                        let filtered_habits: Vec<&habit::Habit> = habits
                            .iter()
                            .filter(|h| match app_state.current_tab {
                                0 => h.get_frequency() == habit::Frequency::Daily,
                                1 => h.get_frequency() == habit::Frequency::Weekly,
                                2 => h.get_frequency() == habit::Frequency::Monthly,
                                _ => false,
                            })
                            .collect();
                        app_state.next(filtered_habits.len());
                    }
                    KeyCode::Tab => {
                        app_state.current_tab = (app_state.current_tab + 1) % 3;
                        app_state.selected = None;
                    }
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
            }
        }
    }
}
