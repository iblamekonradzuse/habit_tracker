use crate::habit::Habit;
use chrono::Local;
use std::io::{self, Write};

pub enum UserAction {
    AddHabit,
    MarkHabit,
    ViewHabits,
    ViewStreaks,
    Quit,
}

pub fn get_user_action() -> UserAction {
    loop {
        println!("\nWhat would you like to do?");
        println!("1. Add a new habit");
        println!("2. Mark a habit as completed");
        println!("3. View habits");
        println!("4. View streaks");
        println!("5. Quit");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => return UserAction::AddHabit,
            "2" => return UserAction::MarkHabit,
            "3" => return UserAction::ViewHabits,
            "4" => return UserAction::ViewStreaks,
            "5" => return UserAction::Quit,
            _ => println!("Invalid input, please try again."),
        }
    }
}

pub fn get_new_habit_info() -> Habit {
    print!("Enter the name of the new habit: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    Habit::new(name.trim().to_string())
}

pub fn display_habits(habits: &[Habit]) {
    println!("\nYour habits:");
    for (i, habit) in habits.iter().enumerate() {
        println!("{}. {}", i + 1, habit.name);
    }
}

pub fn get_habit_index(habits: &[Habit]) -> Option<usize> {
    print!("Enter the number of the habit to mark: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let index: usize = input.trim().parse().ok()?;
    if index > 0 && index <= habits.len() {
        Some(index - 1)
    } else {
        println!("Invalid habit number.");
        None
    }
}

pub fn display_streaks(habits: &[Habit]) {
    let today = Local::now().date_naive();
    println!("\nHabit Streaks:");
    for habit in habits {
        println!("{}: {} days", habit.name, habit.get_streak(today));
    }
}
