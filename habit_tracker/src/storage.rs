use crate::habit::Habit;
use crate::todo::Todo;
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

const HABITS_FILE: &str = "habits.json";
const TODOS_FILE: &str = "todos.json";

pub fn load_habits() -> io::Result<Vec<Habit>> {
    load_data(HABITS_FILE)
}

pub fn save_habits(habits: &[Habit]) -> io::Result<()> {
    save_data(HABITS_FILE, habits)
}

pub fn load_todos() -> io::Result<Vec<Todo>> {
    load_data(TODOS_FILE)
}

pub fn save_todos(todos: &[Todo]) -> io::Result<()> {
    save_data(TODOS_FILE, todos)
}

fn load_data<T: serde::de::DeserializeOwned>(file_name: &str) -> io::Result<Vec<T>> {
    let path = Path::new(file_name);
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(Vec::new())
    }
}

fn save_data<T: serde::Serialize>(file_name: &str, data: &[T]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)?;

    let json = serde_json::to_string_pretty(data)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

