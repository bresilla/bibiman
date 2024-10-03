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

use std::io;

use crate::backend::cliargs::{self, CLIArgs};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::backend::{bib::*, search::BibiSearch};
use crate::{
    frontend::handler::handle_key_events,
    frontend::tui::{Event, Tui},
};
use std::{error, net::SocketAddr};

use arboard::Clipboard;
use color_eyre::eyre::{Ok, Result};
use itertools::Itertools;
use ratatui::widgets::{ListState, TableState};

use super::tui;

// Application result type.
// pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// Areas in which actions are possible
#[derive(Debug)]
pub enum CurrentArea {
    EntryArea,
    TagArea,
    SearchArea,
    HelpArea,
}

// Check which area was active when popup set active
#[derive(Debug)]
pub enum FormerArea {
    EntryArea,
    TagArea,
    SearchArea,
}

// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // // tui initialization
    // pub tui: Tui,
    // main bibliography
    pub main_biblio: BibiMain,
    // bibliographic data
    pub biblio_data: BibiData,
    // search struct:
    pub search_struct: BibiSearch,
    // tag list
    pub tag_list: TagList,
    // table items
    pub entry_table: EntryTable,
    // scroll state info buffer
    pub scroll_info: u16,
    // area
    pub current_area: CurrentArea,
    // mode for popup window
    pub former_area: Option<FormerArea>,
    // search string
    // pub search_string: String,
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
    pub keyword: String,
}

// Function to process inputed characters and convert them (to string, or more complex function)
impl TagListItem {
    pub fn new(info: &str) -> Self {
        Self {
            keyword: info.to_string(),
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
            .map(|i| EntryTableItem::new(&i[0], &i[1], &i[2], &i[3], &i[4], &i[5]))
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
    pub keywords: String,
    pub citekey: String,
    // pub year: u16,
}

impl EntryTableItem {
    pub fn new(
        authors: &str,
        title: &str,
        year: &str,
        pubtype: &str,
        keywords: &str,
        citekey: &str,
    ) -> Self {
        Self {
            authors: authors.to_string(),
            title: title.to_string(),
            year: year.to_string(),
            pubtype: pubtype.to_string(),
            keywords: keywords.to_string(),
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

// impl Default for App {
//     fn default() -> Self {
//         let running = true;
//         let main_biblio = BibiMain::new();
//         let biblio_data = BibiData::new(&main_biblio.bibliography, &main_biblio.citekeys);
//         let tag_list = TagList::from_iter(main_biblio.keyword_list.clone());
//         let search_struct = BibiSearch::default();
//         let entry_table = EntryTable::from_iter(biblio_data.entry_list.bibentries.clone());
//         let current_area = CurrentArea::EntryArea;
//         Self {
//             running,
//             main_biblio,
//             biblio_data,
//             tag_list,
//             search_struct,
//             entry_table,
//             scroll_info: 0,
//             current_area,
//             former_area: None,
//             // search_string: String::new(),
//         }
//     }
// }

impl App {
    // Constructs a new instance of [`App`].
    pub fn new() -> Result<Self> {
        // Self::default()
        let running = true;
        // let tui = Tui::new()?;
        let main_biblio = BibiMain::new();
        let biblio_data = BibiData::new(&main_biblio.bibliography, &main_biblio.citekeys);
        let tag_list = TagList::from_iter(main_biblio.keyword_list.clone());
        let search_struct = BibiSearch::default();
        let entry_table = EntryTable::from_iter(biblio_data.entry_list.bibentries.clone());
        let current_area = CurrentArea::EntryArea;
        Ok(Self {
            running,
            // tui,
            main_biblio,
            biblio_data,
            tag_list,
            search_struct,
            entry_table,
            scroll_info: 0,
            current_area,
            former_area: None,
            // search_string: String::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Initialize the terminal user interface.
        // let backend = CrosstermBackend::new(io::stdout());
        // let terminal = Terminal::new(backend)?;
        // let events = EventHandler::new(250);
        let mut tui = tui::Tui::new()?;
        tui.enter()?;

        // Start the main loop.
        while self.running {
            // Render the user interface.
            tui.draw(self)?;
            // Handle events.
            match tui.next().await? {
                Event::Tick => self.tick(),
                Event::Key(key_event) => handle_key_events(key_event, self, &mut tui)?,
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
            }
        }

        // Exit the user interface.
        tui.exit()?;
        Ok(())
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    // General commands

    // Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    // Toggle moveable list between entries and tags
    pub fn toggle_area(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            self.current_area = CurrentArea::TagArea;
            self.tag_list.tag_list_state.select(Some(0))
        } else if let CurrentArea::TagArea = self.current_area {
            self.current_area = CurrentArea::EntryArea;
            self.tag_list.tag_list_state.select(None)
        }
        // match self.current_area {
        //     CurrentArea::EntryArea => {
        //         self.current_area = CurrentArea::TagArea;
        //         self.tag_list.tag_list_state.select(Some(0))
        //     }
        //     CurrentArea::TagArea => {
        //         self.current_area = CurrentArea::EntryArea;
        //         self.tag_list.tag_list_state.select(None)
        //     }
        //     CurrentArea::SearchArea => {
        //         if let Some(former_area) = &self.former_area {
        //             match former_area {
        //                 FormerArea::EntryArea => self.current_area = CurrentArea::EntryArea,
        //                 FormerArea::TagArea => self.current_area = CurrentArea::TagArea,
        //                 _ => {}
        //             }
        //         }
        //     }
        //     CurrentArea::HelpArea => {
        //         if let Some(former_area) = &self.former_area {
        //             match former_area {
        //                 FormerArea::EntryArea => self.current_area = CurrentArea::EntryArea,
        //                 FormerArea::TagArea => self.current_area = CurrentArea::TagArea,
        //                 FormerArea::SearchArea => self.current_area = CurrentArea::SearchArea,
        //             }
        //         }
        //     }
        // }
    }

    pub fn reset_current_list(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            self.entry_table =
                EntryTable::from_iter(self.biblio_data.entry_list.bibentries.clone());
            if self.search_struct.inner_search {
                self.tag_list = TagList::from_iter(self.main_biblio.keyword_list.clone())
            }
        } else if let CurrentArea::TagArea = self.current_area {
            self.tag_list = TagList::from_iter(self.main_biblio.keyword_list.clone());
            self.tag_list.tag_list_state.select(Some(0))
        }
        self.former_area = None
    }

    // Yank the passed string to system clipboard
    pub fn yank_text(selection: &str) {
        let mut clipboard = Clipboard::new().unwrap();
        let yanked_text = selection.to_string();
        clipboard.set_text(yanked_text).unwrap();
    }

    pub fn scroll_info_down(&mut self) {
        self.scroll_info = self.scroll_info + 1;
    }

    pub fn scroll_info_up(&mut self) {
        if self.scroll_info == 0 {
            {}
        } else {
            self.scroll_info = self.scroll_info - 1;
        }
    }

    // Search Area

    // Enter the search area
    pub fn enter_search_area(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            if let Some(FormerArea::TagArea) = self.former_area {
                self.search_struct.inner_search = true
            }
            self.former_area = Some(FormerArea::EntryArea)
        } else if let CurrentArea::TagArea = self.current_area {
            self.former_area = Some(FormerArea::TagArea)
        }
        self.current_area = CurrentArea::SearchArea
    }

    // Confirm search: Search former list by pattern
    pub fn confirm_search(&mut self) {
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.current_area = CurrentArea::EntryArea;
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.current_area = CurrentArea::TagArea;
            self.tag_list.tag_list_state.select(Some(0))
        }
        self.former_area = Some(FormerArea::SearchArea);
        self.search_struct.search_string.clear();
    }

    // Break search: leave search area without filtering list
    pub fn break_search(&mut self) {
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.current_area = CurrentArea::EntryArea;
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.current_area = CurrentArea::TagArea;
            self.tag_list.tag_list_state.select(Some(0))
        }
        self.reset_current_list();
        self.former_area = None;
        // If search is canceled, reset default status of struct
        self.search_struct.search_string.clear();
    }

    // Search entry list
    pub fn search_entries(&mut self) {
        let orig_list = {
            if self.search_struct.inner_search {
                let orig_list = &self.search_struct.filtered_entry_list;
                orig_list
            } else {
                let orig_list = &self.biblio_data.entry_list.bibentries;
                orig_list
            }
        };
        let filtered_list =
            BibiSearch::search_entry_list(&mut self.search_struct.search_string, orig_list.clone());
        //search::search_entry_list(&self.search_string, orig_list.clone());
        self.entry_table = EntryTable::from_iter(filtered_list)
    }

    // Remove last char from search pattern and filter list immidiately
    pub fn search_pattern_pop(&mut self) {
        self.search_struct.search_string.pop();
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.search_entries();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.search_tags();
        }
    }

    // Add current char to search pattern and filter list immidiatley
    pub fn search_pattern_push(&mut self, search_pattern: char) {
        self.search_struct.search_string.push(search_pattern);
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.search_entries();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.search_tags();
        }
    }

    // Tag List commands

    // Movement
    pub fn select_next_tag(&mut self) {
        self.tag_list.tag_list_state.select_next();
    }

    pub fn select_previous_tag(&mut self) {
        self.tag_list.tag_list_state.select_previous();
    }

    pub fn select_first_tag(&mut self) {
        self.tag_list.tag_list_state.select_first();
    }

    pub fn select_last_tag(&mut self) {
        self.tag_list.tag_list_state.select_last();
    }

    pub fn get_selected_tag(&self) -> &str {
        let idx = self.tag_list.tag_list_state.selected().unwrap();
        let keyword = &self.tag_list.tag_list_items[idx].keyword;
        keyword
    }

    pub fn search_tags(&mut self) {
        let orig_list = &self.main_biblio.keyword_list;
        let filtered_list =
            BibiSearch::search_tag_list(&self.search_struct.search_string, orig_list.clone());
        self.tag_list = TagList::from_iter(filtered_list)
    }

    // Filter the entry list by tags
    // If already inside a filtered tag or entry list, apply the filtering
    // to the already filtered list only
    pub fn filter_for_tags(&mut self) {
        let orig_list = {
            if self.search_struct.inner_search {
                let orig_list = &self.search_struct.filtered_entry_list;
                orig_list
            } else {
                let orig_list = &self.biblio_data.entry_list.bibentries;
                orig_list
            }
        };
        let keyword = self.get_selected_tag();
        let filtered_list = BibiSearch::filter_entries_by_tag(&keyword, &orig_list);
        self.search_struct.filtered_entry_list = filtered_list;
        self.entry_table = EntryTable::from_iter(self.search_struct.filtered_entry_list.clone());
        self.toggle_area();
        self.former_area = Some(FormerArea::TagArea);
    }

    // Entry Table commands

    // Movement
    pub fn select_next_entry(&mut self) {
        self.scroll_info = 0;
        self.entry_table.entry_table_state.select_next();
    }

    pub fn select_previous_entry(&mut self) {
        self.scroll_info = 0;
        self.entry_table.entry_table_state.select_previous();
    }

    pub fn select_first_entry(&mut self) {
        self.scroll_info = 0;
        self.entry_table.entry_table_state.select_first();
    }

    pub fn select_last_entry(&mut self) {
        self.scroll_info = 0;
        self.entry_table.entry_table_state.select_last();
    }

    // Get the citekey of the selected entry
    pub fn get_selected_citekey(&self) -> &str {
        let idx = self.entry_table.entry_table_state.selected().unwrap();
        let citekey = &self.entry_table.entry_table_items[idx].citekey;
        citekey
    }

    pub fn run_editor(&mut self, tui: &mut Tui) -> Result<()> {
        tui.exit()?;
        let cmd = String::from("hx");
        let args: Vec<String> = vec!["test.bib".into()];
        let status = std::process::Command::new(&cmd).args(&args).status()?;
        if !status.success() {
            eprintln!("Spawning editor failed with status {}", status);
        }
        tui.enter()?;
        tui.terminal.clear()?;
        Ok(())
    }
}
