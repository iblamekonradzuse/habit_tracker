use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Habit {
    pub name: String,
    completed_dates: Vec<NaiveDate>,
}

impl Habit {
    pub fn new(name: String) -> Self {
        Habit {
            name,
            completed_dates: Vec::new(),
        }
    }

    pub fn mark_completed(&mut self, date: NaiveDate) {
        if !self.completed_dates.contains(&date) {
            self.completed_dates.push(date);
            self.completed_dates.sort_unstable();
        }
    }

    pub fn is_completed(&self, date: NaiveDate) -> bool {
        self.completed_dates.contains(&date)
    }

    pub fn get_streak(&self, end_date: NaiveDate) -> u32 {
        let mut streak = 0;
        let mut current_date = end_date;

        while let Some(last_completed) = self
            .completed_dates
            .iter()
            .rev()
            .find(|&&d| d <= current_date)
        {
            if *last_completed == current_date {
                streak += 1;
                current_date = current_date.pred_opt().unwrap_or(current_date);
            } else {
                break;
            }
        }

        streak
    }

    pub fn get_completion_status(&self, start_date: NaiveDate, end_date: NaiveDate) -> Vec<bool> {
        let mut status = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            status.push(self.is_completed(current_date));
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        status
    }
}

