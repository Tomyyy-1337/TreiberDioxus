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

    pub fn remove_entry(&mut self, index: usize) {
        self.entries.retain(|entry| entry.index != index);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indexed<T> {
    pub index: usize,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub enum OverlayElementTypes {
    CurrentSpeed,
    CurrentBPM,
    SessionDistance,
    SessionTime,
    Split500m,
    DistancePerStroke,
    FahrtSchlagCount
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub struct OverlayElement {
    pub element_type: OverlayElementTypes,
    pub visible: bool,
}

impl OverlayElement {
    pub fn new(element_type: OverlayElementTypes, visible: bool) -> Self {
        OverlayElement {
            element_type,
            visible
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OverlayPosition {
    Top,
    Bottom
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverlaySettings {
    show_overlay: bool,
    overlay_position: OverlayPosition,
    elements: [OverlayElement; 6],
}

impl OverlaySettings {
    pub fn new() -> Self {
        OverlaySettings {
            show_overlay: true,
            overlay_position: OverlayPosition::Top,
            elements: [
                OverlayElement::new(OverlayElementTypes::CurrentSpeed, true),
                OverlayElement::new(OverlayElementTypes::Split500m, true),
                OverlayElement::new(OverlayElementTypes::CurrentBPM, true),
                OverlayElement::new(OverlayElementTypes::DistancePerStroke, true),
                OverlayElement::new(OverlayElementTypes::SessionDistance, true),
                OverlayElement::new(OverlayElementTypes::SessionTime, true),
            ],
        }
    }

    pub fn set_element_visibility(&mut self, index: usize, visible: bool) {
        if let Some(element) = self.elements.get_mut(index) {
            element.visible = visible;
        }
    }

    pub fn set_show_overlay(&mut self, show: bool) {
        self.show_overlay = show;
    }

    pub fn set_overlay_position(&mut self, position: OverlayPosition) {
        self.overlay_position = position;
    }
    
    pub fn get_show_overlay(&self) -> bool {
        self.show_overlay
    }

    pub fn get_overlay_position(&self) -> OverlayPosition {
        self.overlay_position
    }

    pub fn get_visible_elements(&self) -> impl Iterator<Item = &OverlayElementTypes> {
        self.elements.iter().filter(|e| e.visible).map(|e| &e.element_type)
    }

    pub fn get_elements(&self) -> impl Iterator<Item = &OverlayElement> {
        self.elements.iter()
    }

    pub fn move_element(&mut self, from: usize, to: usize) {
        if from < self.elements.len() && to < self.elements.len() && from != to {
            if from < to {
                self.elements[from..=to].rotate_left(1);
            } else {
                self.elements[to..=from].rotate_right(1);
            }
        }
    }
}