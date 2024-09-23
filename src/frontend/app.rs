// bibiman - a TUI for managing BibLaTeX databases
// Copyright (C) 2024  lukeflo
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
/////

use crate::backend::bib::*;
use std::error;

use itertools::Itertools;
use ratatui::widgets::{ListState, TableState};

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// Areas in which actions are possible
#[derive(Debug)]
pub enum CurrentArea {
    EntryArea,
    TagArea,
    // SearchArea,
}

// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // list
    pub tag_list: TagList,
    // TODO: table items
    pub entry_table: EntryTable,
    // area
    pub current_area: CurrentArea,
}

// Define the fundamental List
#[derive(Debug)]
pub struct TagList {
    pub tag_list_items: Vec<TagListItem>,
    pub tag_list_state: ListState,
}

// Structure of the list items. Can be a simple string or something more elaborated
// eg:
// struct TagListItem {
//     todo: String,
//     info: String,
//     status: Status,
// }
// where Status has to be defined explicitly somewhere else
#[derive(Debug)]
pub struct TagListItem {
    pub info: String,
}

// Function to process inputed characters and convert them (to string, or more complex function)
impl TagListItem {
    pub fn new(info: &str) -> Self {
        Self {
            info: info.to_string(),
        }
    }
}

// INFO: in the original template it was <&'static str> instead of <String>
impl FromIterator<String> for TagList {
    // INFO: Here to originally <&'static str>
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let tag_list_items = iter
            .into_iter()
            // INFO: here originally not borrowed (without Ampersand'&')
            .map(|info| TagListItem::new(&info))
            .collect();
        let tag_list_state = ListState::default(); // for preselection: .with_selected(Some(0));
        Self {
            tag_list_items,
            tag_list_state,
        }
    }
}

// Iterate over vector fields with entries data to create TableItems
// Number of Strings has to match number of fields
impl FromIterator<[String; 5]> for EntryTable {
    fn from_iter<T: IntoIterator<Item = [String; 5]>>(iter: T) -> Self {
        // Has to be Vev<EntryTableItem>
        let entry_table_items = iter
            .into_iter()
            // fields in map must not be named specific
            .map(|[authors, title, year, pubtype, citekey]| {
                EntryTableItem::new(&authors, &title, &year, &pubtype, &citekey)
            })
            .sorted_by(|a, b| a.authors.cmp(&b.authors))
            .collect();
        let entry_table_state = TableState::default().with_selected(0);
        Self {
            entry_table_items,
            entry_table_state,
        }
    }
}

// Define list containing entries as table
#[derive(Debug)]
pub struct EntryTable {
    pub entry_table_items: Vec<EntryTableItem>,
    pub entry_table_state: TableState,
}

// Define contents of each entry table row
#[derive(Debug)]
pub struct EntryTableItem {
    pub authors: String,
    pub title: String,
    pub year: String,
    pub pubtype: String,
    pub citekey: String,
    // pub year: u16,
}

impl EntryTableItem {
    pub fn new(authors: &str, title: &str, year: &str, pubtype: &str, citekey: &str) -> Self {
        Self {
            authors: authors.to_string(),
            title: title.to_string(),
            year: year.to_string(),
            pubtype: pubtype.to_string(),
            citekey: citekey.to_string(),
        }
    }

    // This functions decides which fields are rendered in the entry table
    // Fields which should be usable but not visible can be left out
    pub fn ref_vec(&self) -> Vec<&String> {
        vec![&self.authors, &self.title, &self.year, &self.pubtype]
    }

    pub fn authors(&self) -> &str {
        &self.authors
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn year(&self) -> &str {
        &self.year
    }

    pub fn pubtype(&self) -> &str {
        &self.pubtype
    }

    pub fn citekey(&self) -> &str {
        &self.citekey
    }
}

impl Default for App {
    fn default() -> Self {
        // TEST: read file
        let lines = BibiMain::new().citekeys;
        let iter = vec![
            [
                "Mrs. Doubtfire".to_string(),
                "A great book of great length".to_string(),
                "2003".to_string(),
                "book".to_string(),
                "doubtfire_2003".to_string(),
            ],
            [
                "Veye Tatah".to_string(),
                "Modern economy".to_string(),
                1995.to_string(),
                "article".to_string(),
                "tatah_1995".to_string(),
            ],
            [
                "Joseph Conrad".to_string(),
                "Heart of Darkness".to_string(),
                1899.to_string(),
                "book".to_string(),
                "conrad_1899".to_string(),
            ],
            [
                "Michelle-Rolpg Trouillot".to_string(),
                "Silencing the Past".to_string(),
                "1995".to_string(),
                "book".to_string(),
                "trouillot_1995".to_string(),
            ],
            [
                "Zora Neale Hurston".to_string(),
                "Barracoon".to_string(),
                "1919".to_string(),
                "book".to_string(),
                "hurston_1919".to_string(),
            ],
        ];
        Self {
            running: true,
            tag_list: TagList::from_iter(lines),
            entry_table: EntryTable::from_iter(iter),
            current_area: CurrentArea::EntryArea,
        }
    }
}

impl App {
    // Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    // Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    // Toggle moveable list between entries and tags
    pub fn toggle_area(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.current_area = CurrentArea::TagArea,
            CurrentArea::TagArea => self.current_area = CurrentArea::EntryArea,
        }
    }

    pub fn select_none(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select(None),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select(None),
        }
        // self.tag_list.tag_list_state.select(None);
    }

    pub fn select_next(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_next(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_next(),
        }
        // self.tag_list.tag_list_state.select_next();
    }
    pub fn select_previous(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_previous(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_previous(),
        }
        // self.tag_list.tag_list_state.select_previous();
    }

    pub fn select_first(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_first(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_first(),
        }
        // self.tag_list.tag_list_state.select_first();
    }

    pub fn select_last(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_last(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_last(),
        }
        // self.tag_list.tag_list_state.select_last();
    }

    // pub fn select_none(&mut self) {
    //     self.entry_table.entry_table_state.select(None);
    // }

    // pub fn select_next(&mut self) {
    //     self.entry_table.entry_table_state.select_next();
    // }
    // pub fn select_previous(&mut self) {
    //     self.entry_table.entry_table_state.select_previous();
    // }

    // pub fn select_first(&mut self) {
    //     self.entry_table.entry_table_state.select_first();
    // }

    // pub fn select_last(&mut self) {
    //     self.entry_table.entry_table_state.select_last();
    // }
}
