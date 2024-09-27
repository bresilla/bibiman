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

use arboard::Clipboard;
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
    // main bibliography
    pub main_biblio: BibiMain,
    // bibliographic data
    pub biblio_data: BibiData,
    // tag list
    pub tag_list: TagList,
    // table items
    pub entry_table: EntryTable,
    // scroll state info buffer
    pub scroll_info: u16,
    // area
    pub current_area: CurrentArea,
}

// Define the fundamental List
#[derive(Debug)]
pub struct TagList {
    pub tag_list_items: Vec<TagListItem>,
    pub tag_list_state: ListState,
}

// Structure of the list items.
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

impl FromIterator<String> for TagList {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let tag_list_items = iter
            .into_iter()
            .map(|info| TagListItem::new(&info))
            .collect();
        let tag_list_state = ListState::default(); // for preselection: .with_selected(Some(0));
        Self {
            tag_list_items,
            tag_list_state,
        }
    }
}

// // Also possible with vector.
impl FromIterator<Vec<String>> for EntryTable {
    fn from_iter<T: IntoIterator<Item = Vec<String>>>(iter: T) -> Self {
        let entry_table_items = iter
            .into_iter()
            .sorted()
            .map(|i| EntryTableItem::new(&i[0], &i[1], &i[2], &i[3], &i[4]))
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
        let running = true;
        let main_biblio = BibiMain::new();
        let biblio_data = BibiData::new(&main_biblio.bibliography, &main_biblio.citekeys);
        let tag_list = TagList::from_iter(main_biblio.citekeys.clone());
        let entry_table = EntryTable::from_iter(biblio_data.entry_list.bibentries.clone());
        let current_area = CurrentArea::EntryArea;
        Self {
            running,
            main_biblio,
            biblio_data,
            tag_list,
            entry_table,
            scroll_info: 0,
            current_area,
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

    pub fn scroll_info_down(&mut self) {
        self.scroll_info = (self.scroll_info + 1);
    }

    pub fn scroll_info_up(&mut self) {
        if self.scroll_info == 0 {
            {}
        } else {
            self.scroll_info = (self.scroll_info - 1);
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
        self.scroll_info = 0;
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_next(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_next(),
        }
        // self.tag_list.tag_list_state.select_next();
    }
    pub fn select_previous(&mut self) {
        self.scroll_info = 0;
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_previous(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_previous(),
        }
        // self.tag_list.tag_list_state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.scroll_info = 0;
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_first(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_first(),
        }
        // self.tag_list.tag_list_state.select_first();
    }

    pub fn select_last(&mut self) {
        self.scroll_info = 0;
        match self.current_area {
            CurrentArea::EntryArea => self.entry_table.entry_table_state.select_last(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_last(),
        }
        // self.tag_list.tag_list_state.select_last();
    }

    // Get the citekey of the selected entry
    pub fn get_selected_citekey(&self) -> &str {
        let idx = self.entry_table.entry_table_state.selected().unwrap();
        let citekey = &self.entry_table.entry_table_items[idx].citekey;
        citekey
    }

    // Yank the passed string to system clipboard
    pub fn yank_text(selection: &str) {
        let mut clipboard = Clipboard::new().unwrap();
        let yanked_text = selection.to_string();
        clipboard.set_text(yanked_text).unwrap();
    }
}
