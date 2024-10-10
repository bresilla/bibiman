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

use super::tui;
use crate::backend::cliargs::CLIArgs;
use crate::backend::{bib::*, search::BibiSearch};
use crate::{
    frontend::entries::EntryTable, frontend::handler::handle_key_events,
    frontend::keywords::TagList, frontend::tui::Event,
};
use arboard::Clipboard;
use color_eyre::eyre::{Ok, Result};
use std::path::PathBuf;

// Areas in which actions are possible
#[derive(Debug)]
pub enum CurrentArea {
    EntryArea,
    TagArea,
    SearchArea,
    HelpArea,
    InfoArea,
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
    // main bib file
    pub main_bibfile: PathBuf,
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
}

impl App {
    // Constructs a new instance of [`App`].
    pub fn new(args: CLIArgs) -> Result<Self> {
        // Self::default()
        let running = true;
        let main_bibfile = args.bibfilearg;
        let main_biblio = BibiMain::new(main_bibfile.clone());
        let biblio_data = BibiData::new(&main_biblio.bibliography, &main_biblio.citekeys);
        let tag_list = TagList::from_iter(main_biblio.keyword_list.clone());
        let search_struct = BibiSearch::default();
        let entry_table = EntryTable::from_iter(biblio_data.entry_list.bibentries.clone());
        let current_area = CurrentArea::EntryArea;
        Ok(Self {
            running,
            main_bibfile,
            main_biblio,
            biblio_data,
            tag_list,
            search_struct,
            entry_table,
            scroll_info: 0,
            current_area,
            former_area: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
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

    pub fn update_lists(&mut self) {
        self.main_biblio = BibiMain::new(self.main_bibfile.clone());
        self.biblio_data =
            BibiData::new(&self.main_biblio.bibliography, &self.main_biblio.citekeys);
        self.tag_list = TagList::from_iter(self.main_biblio.keyword_list.clone());
        self.entry_table = EntryTable::from_iter(self.biblio_data.entry_list.bibentries.clone());
    }

    // Toggle moveable list between entries and tags
    pub fn toggle_area(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            self.entry_table.entry_scroll_state = self.entry_table.entry_scroll_state.position(0);
            self.current_area = CurrentArea::TagArea;
            self.tag_list.tag_list_state.select(Some(0));
            self.tag_list.tag_scroll_state = self
                .tag_list
                .tag_scroll_state
                .position(self.tag_list.tag_list_state.selected().unwrap());
        } else if let CurrentArea::TagArea = self.current_area {
            self.current_area = CurrentArea::EntryArea;
            self.tag_list.tag_list_state.select(None);
            self.entry_table.entry_scroll_state = self
                .entry_table
                .entry_scroll_state
                .position(self.entry_table.entry_table_state.selected().unwrap());
        }
    }

    pub fn reset_current_list(&mut self) {
        self.entry_table = EntryTable::from_iter(self.biblio_data.entry_list.bibentries.clone());
        self.tag_list = TagList::from_iter(self.main_biblio.keyword_list.clone());
        if let CurrentArea::TagArea = self.current_area {
            self.tag_list.tag_list_state.select(Some(0))
        }
        self.search_struct.filtered_entry_list.clear();
        self.search_struct.filtered_tag_list.clear();
        self.search_struct.inner_search = false;
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
        // But keep filtering by tag if applied before entering search area
        if !self.search_struct.inner_search {
            self.reset_current_list();
        }
        self.former_area = None;
        // If search is canceled, reset default status of struct
        self.search_struct.search_string.clear();
    }

    // Remove last char from search pattern and filter list immidiately
    pub fn search_pattern_pop(&mut self) {
        self.search_struct.search_string.pop();
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.search_entries();
            self.filter_tags_by_entries();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.search_tags();
        }
    }

    // Add current char to search pattern and filter list immidiatley
    pub fn search_pattern_push(&mut self, search_pattern: char) {
        self.search_struct.search_string.push(search_pattern);
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.search_entries();
            self.filter_tags_by_entries();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.search_tags();
        }
    }
}
