use chrono::Datelike;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Frequency::Daily => write!(f, "Daily"),
            Frequency::Weekly => write!(f, "Weekly"),
            Frequency::Monthly => write!(f, "Monthly"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Habit {
    pub name: String,
    pub category: String,
    pub frequency: Frequency,
    completed_dates: Vec<NaiveDate>,
}

impl Habit {
    pub fn new(name: String, category: String, frequency: Frequency) -> Self {
        Habit {
            name,
            category,
            frequency,
            completed_dates: Vec::new(),
        }
    }

    pub fn mark_completed(&mut self, date: NaiveDate) {
        if !self.completed_dates.contains(&date) {
            self.completed_dates.push(date);
            self.completed_dates.sort_unstable();
        }
    }

    pub fn unmark_completed(&mut self, date: NaiveDate) {
        self.completed_dates.retain(|&d| d != date);
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
            match self.frequency {
                Frequency::Daily => {
                    if *last_completed == current_date {
                        streak += 1;
                        current_date = current_date.pred_opt().unwrap_or(current_date);
                    } else {
                        break;
                    }
                }
                Frequency::Weekly => {
                    if last_completed.iso_week() == current_date.iso_week() {
                        streak += 1;
                        current_date = current_date - chrono::Duration::weeks(1);
                    } else {
                        break;
                    }
                }
                Frequency::Monthly => {
                    if last_completed.year() == current_date.year()
                        && last_completed.month() == current_date.month()
                    {
                        streak += 1;
                        current_date = current_date.with_day(1).unwrap_or(current_date)
                            - chrono::Duration::days(1);
                    } else {
                        break;
                    }
                }
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

    pub fn get_frequency(&self) -> Frequency {
        self.frequency
    }

    pub fn get_current_streak(&self, date: NaiveDate) -> u32 {
        self.get_streak(date)
    }
}
