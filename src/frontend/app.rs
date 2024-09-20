use crate::backend::bib::*;
use std::error;

use ratatui::widgets::ListState;

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
    pub entry_list: EntryList,
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

impl FromIterator<(String, String)> for EntryList {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let entry_list_items = iter
            .into_iter()
            .map(|(authors, title)| EntryListItem::new(&authors, &title))
            .collect();
        let entry_list_state = ListState::default();
        Self {
            entry_list_items,
            entry_list_state,
        }
    }
}

// Define list containing entries as table
#[derive(Debug)]
pub struct EntryList {
    pub entry_list_items: Vec<EntryListItem>,
    pub entry_list_state: ListState,
}

// Define contents of each entry table row
#[derive(Debug)]
pub struct EntryListItem {
    pub authors: String,
    pub title: String,
    // pub year: u16,
}

impl EntryListItem {
    pub fn new(authors: &str, title: &str) -> Self {
        Self {
            authors: authors.to_string(),
            title: title.to_string(),
        }
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
        // let mylist = ["Item 1", "Item 2"];
        Self {
            running: true,
            // INFO: here the function(s) for creating the list has to be placed inside the parantheses -> Bib::whatever
            tag_list: TagList::from_iter(lines),
            entry_list: EntryList::from_iter(iter),
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
            CurrentArea::EntryArea => self.entry_list.entry_list_state.select(None),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select(None),
        }
        // self.tag_list.tag_list_state.select(None);
    }

    pub fn select_next(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_list.entry_list_state.select_next(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_next(),
        }
        // self.tag_list.tag_list_state.select_next();
    }
    pub fn select_previous(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_list.entry_list_state.select_previous(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_previous(),
        }
        // self.tag_list.tag_list_state.select_previous();
    }

    pub fn select_first(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_list.entry_list_state.select_first(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_first(),
        }
        // self.tag_list.tag_list_state.select_first();
    }

    pub fn select_last(&mut self) {
        match self.current_area {
            CurrentArea::EntryArea => self.entry_list.entry_list_state.select_last(),
            CurrentArea::TagArea => self.tag_list.tag_list_state.select_last(),
        }
        // self.tag_list.tag_list_state.select_last();
    }

    // pub fn select_none(&mut self) {
    //     self.entry_list.entry_list_state.select(None);
    // }

    // pub fn select_next(&mut self) {
    //     self.entry_list.entry_list_state.select_next();
    // }
    // pub fn select_previous(&mut self) {
    //     self.entry_list.entry_list_state.select_previous();
    // }

    // pub fn select_first(&mut self) {
    //     self.entry_list.entry_list_state.select_first();
    // }

    // pub fn select_last(&mut self) {
    //     self.entry_list.entry_list_state.select_last();
    // }
}
