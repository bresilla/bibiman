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

impl FromIterator<(String, String)> for EntryTable {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        // Has to be Vev<EntryTableItem>
        let entry_table_items = iter
            .into_iter()
            // .map(|(authors, title)| EntryTableItem::new(&authors, &title))
            .map(|(authors, title)| EntryTableItem::new(&authors, &title))
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
    // pub year: u16,
}

impl EntryTableItem {
    pub fn new(authors: &str, title: &str) -> Self {
        Self {
            authors: authors.to_string(),
            title: title.to_string(),
        }
    }

    pub fn ref_vec(&self) -> Vec<&String> {
        vec![&self.authors, &self.title]
    }

    pub fn authors(&self) -> &str {
        &self.authors
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl Default for App {
    fn default() -> Self {
        // TEST: read file
        let lines = Bibi::new().citekeys;
        let iter = vec![
            (
                "Mrs. Doubtfire".to_string(),
                "A great book of great length".to_string(),
            ),
            ("Veye Tatah".to_string(), "Modern economy".to_string()),
            ("Joseph Conrad".to_string(), "Heart of Darkness".to_string()),
            (
                "Michelle-Rolpg Trouillot".to_string(),
                "Silencing the Past".to_string(),
            ),
            ("Zora Neale Hurston".to_string(), "Barracoon".to_string()),
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
