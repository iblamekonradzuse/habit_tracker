use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod cli;
mod habit;
mod storage;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut habits = storage::load_habits()?;
    let mut current_date = chrono::Local::now().date_naive();
    let mut app_state = ui::AppState::default();

    let res = run_app(
        &mut terminal,
        &mut habits,
        &mut current_date,
        &mut app_state,
    );

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

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
                        app_state.input_mode = ui::InputMode::AddingHabit;
                        app_state.new_habit_name.clear();
                    }
                    KeyCode::Enter => {
                        if let Some(index) = app_state.selected {
                            if habits[index].is_completed(*current_date) {
                                habits[index].unmark_completed(*current_date);
                            } else {
                                habits[index].mark_completed(*current_date);
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(index) = app_state.selected {
                            habits.remove(index);
                            if index >= habits.len() {
                                app_state.selected = habits.len().checked_sub(1);
                            }
                        }
                    }
                    KeyCode::Left => {
                        *current_date = current_date.pred_opt().unwrap_or(*current_date)
                    }
                    KeyCode::Right => {
                        *current_date = current_date.succ_opt().unwrap_or(*current_date)
                    }
                    KeyCode::Up => app_state.previous(),
                    KeyCode::Down => app_state.next(habits.len()),
                    _ => {}
                },
                ui::InputMode::AddingHabit => match key.code {
                    KeyCode::Enter => {
                        let new_habit = habit::Habit::new(app_state.new_habit_name.clone());
                        habits.push(new_habit);
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_habit_name.clear();
                    }
                    KeyCode::Esc => {
                        app_state.input_mode = ui::InputMode::Normal;
                        app_state.new_habit_name.clear();
                    }
                    KeyCode::Char(c) => {
                        app_state.new_habit_name.push(c);
                    }
                    KeyCode::Backspace => {
                        app_state.new_habit_name.pop();
                    }
                    _ => {}
                },
            }
        }
    }
}

