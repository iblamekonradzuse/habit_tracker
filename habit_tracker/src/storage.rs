use crate::habit::Habit;
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

const STORAGE_FILE: &str = "habits.json";

pub fn load_habits() -> io::Result<Vec<Habit>> {
    let path = Path::new(STORAGE_FILE);
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        Ok(Vec::new())
    }
}

pub fn save_habits(habits: &[Habit]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(STORAGE_FILE)?;

    let json = serde_json::to_string_pretty(habits)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

