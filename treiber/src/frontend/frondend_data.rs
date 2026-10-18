use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Display};

use crate::{FinishedFahrt};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "light"),
            Theme::Dark => write!(f, "dark"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoRotate {
    pub enabled: bool,
}

const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fahrtenbuch {
    pub next_index: usize,
    pub entries: VecDeque<Indexed<FinishedFahrt>>,
}

impl Fahrtenbuch {
    pub fn new() -> Self {
        Fahrtenbuch {
            next_index: 0,
            entries: VecDeque::new(),
        }
    }

    pub fn add_entry(&mut self, value: FinishedFahrt) {
        let entry = Indexed {
            index: self.next_index,
            value,
        };
        self.entries.push_back(entry);
        self.next_index += 1;
        while self.entries.len() > MAX_ENTRIES {
            self.entries.pop_front();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indexed<T> {
    pub index: usize,
    pub value: T,
}
