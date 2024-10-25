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

use crate::bibiman::entries::EntryTableColumn;
use crate::bibiman::{bibisetup::*, search::BibiSearch};
use crate::cliargs::CLIArgs;
use crate::tui::Tui;
use crate::{bibiman::entries::EntryTable, bibiman::keywords::TagList};
use arboard::Clipboard;
use color_eyre::eyre::{Context, Ok, Result};
use core::panic;
use editor_command::EditorBuilder;
use ratatui::widgets::ScrollbarState;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tui_input::Input;

pub mod bibisetup;
pub mod entries;
pub mod keywords;
pub mod search;

// Areas in which actions are possible
#[derive(Debug)]
pub enum CurrentArea {
    EntryArea,
    TagArea,
    SearchArea,
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
pub struct Bibiman {
    // main bib file
    pub main_bibfile: PathBuf,
    // main bibliography
    pub main_biblio: BibiSetup,
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

impl Bibiman {
    // Constructs a new instance of [`App`].
    pub fn new(args: CLIArgs) -> Result<Self> {
        let main_bibfile = args.bibfilearg;
        let main_biblio = BibiSetup::new(main_bibfile.clone());
        let tag_list = TagList::new(main_biblio.keyword_list.clone());
        let search_struct = BibiSearch::default();
        let entry_table = EntryTable::new(main_biblio.entry_list.clone());
        let current_area = CurrentArea::EntryArea;
        Ok(Self {
            main_bibfile,
            main_biblio,
            tag_list,
            search_struct,
            entry_table,
            scroll_info: 0,
            current_area,
            former_area: None,
        })
    }

    pub fn update_lists(&mut self) {
        self.main_biblio = BibiSetup::new(self.main_bibfile.clone());
        // self.tag_list = TagList::from_iter(self.main_biblio.keyword_list.clone());
        self.tag_list = TagList::new(self.main_biblio.keyword_list.clone());
        self.entry_table = EntryTable::new(self.main_biblio.entry_list.clone());
    }

    /// Toggle moveable list between entries and tags
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
        self.entry_table = EntryTable::new(self.main_biblio.entry_list.clone());
        self.tag_list = TagList::new(self.main_biblio.keyword_list.clone());
        if let CurrentArea::TagArea = self.current_area {
            self.tag_list.tag_list_state.select(Some(0))
        }
        self.entry_table.entry_table_at_search_start.clear();
        self.search_struct.filtered_tag_list.clear();
        self.search_struct.inner_search = false;
        self.former_area = None
    }

    /// Yank the passed string to system clipboard
    pub fn yank_text(selection: &str) {
        let mut clipboard = Clipboard::new().unwrap();
        let yanked_text = selection.to_string();
        clipboard.set_text(yanked_text).unwrap();
    }

    pub fn scroll_info_down(&mut self) {
        self.entry_table.entry_info_scroll = self.entry_table.entry_info_scroll.saturating_add(1);
        self.entry_table.entry_info_scroll_state = self
            .entry_table
            .entry_info_scroll_state
            .position(self.entry_table.entry_info_scroll.into());
    }

    pub fn scroll_info_up(&mut self) {
        self.entry_table.entry_info_scroll = self.entry_table.entry_info_scroll.saturating_sub(1);
        self.entry_table.entry_info_scroll_state = self
            .entry_table
            .entry_info_scroll_state
            .position(self.entry_table.entry_info_scroll.into());
    }
}

impl Bibiman {
    // Entry Table commands

    /// Select next entry in Table holding the bibliographic entries.
    ///
    /// Takes u16 value as argument to specify number of entries which
    /// should be scrolled
    pub fn select_next_entry(&mut self, entries: u16) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        self.entry_table.entry_table_state.scroll_down_by(entries);
        self.entry_table.entry_scroll_state = self
            .entry_table
            .entry_scroll_state
            .position(self.entry_table.entry_table_state.selected().unwrap());
    }

    /// Select previous entry in Table holding the bib entries.
    ///
    /// Takes u16 value as argument to specify number of entries which
    /// should be scrolled
    pub fn select_previous_entry(&mut self, entries: u16) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        self.entry_table.entry_table_state.scroll_up_by(entries);
        self.entry_table.entry_scroll_state = self
            .entry_table
            .entry_scroll_state
            .position(self.entry_table.entry_table_state.selected().unwrap());
    }

    /// Select first entry in bib list
    pub fn select_first_entry(&mut self) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        self.entry_table.entry_table_state.select_first();
        self.entry_table.entry_scroll_state = self.entry_table.entry_scroll_state.position(0);
    }

    /// Select last entry in bib list
    pub fn select_last_entry(&mut self) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        // self.entry_table.entry_table_state.select_last(); // Does not work properly after upgrading to ratatui 0.29.0
        self.entry_table
            .entry_table_state
            .select(Some(self.entry_table.entry_table_items.len() - 1));
        self.entry_table.entry_scroll_state = self
            .entry_table
            .entry_scroll_state
            .position(self.entry_table.entry_table_items.len());
    }

    /// Select next (right) column of entry table
    pub fn select_next_column(&mut self) {
        if self
            .entry_table
            .entry_table_state
            .selected_column()
            .unwrap()
            == 3
        {
            self.entry_table.entry_table_state.select_first_column();
        } else {
            self.entry_table.entry_table_state.select_next_column();
        }
        match self.entry_table.entry_table_selected_column {
            EntryTableColumn::Authors => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Title;
            }
            EntryTableColumn::Title => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Year;
            }
            EntryTableColumn::Year => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Pubtype;
            }
            EntryTableColumn::Pubtype => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Authors;
            }
        }
    }

    /// Select previous (left) column of entry table
    pub fn select_prev_column(&mut self) {
        if self
            .entry_table
            .entry_table_state
            .selected_column()
            .unwrap()
            == 0
        {
            self.entry_table.entry_table_state.select_last_column();
        } else {
            self.entry_table.entry_table_state.select_previous_column();
        }
        match self.entry_table.entry_table_selected_column {
            EntryTableColumn::Authors => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Pubtype;
            }
            EntryTableColumn::Title => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Authors;
            }
            EntryTableColumn::Year => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Title;
            }
            EntryTableColumn::Pubtype => {
                self.entry_table.entry_table_selected_column = EntryTableColumn::Year;
            }
        }
    }

    // Get the citekey of the selected entry
    pub fn get_selected_citekey(&self) -> &str {
        let idx = self.entry_table.entry_table_state.selected().unwrap();
        let citekey = &self.entry_table.entry_table_items[idx].citekey;
        citekey
    }

    pub fn run_editor(&mut self, tui: &mut Tui) -> Result<()> {
        // get filecontent and citekey for calculating line number
        let citekey = self.get_selected_citekey();
        // create independent copy of citekey for finding entry after updating list
        let saved_key = citekey.to_owned();
        let filepath = self.main_biblio.bibfile.display().to_string();
        let filecontent = self.main_biblio.bibfilestring.clone();
        let mut line_count = 0;

        for line in filecontent.lines() {
            line_count = line_count + 1;
            // if reaching the citekey break the loop
            // if reaching end of lines without match, reset to 0
            if line.contains(&citekey) {
                break;
            } else if line_count == filecontent.len() {
                eprintln!(
                    "Citekey {} not found, opening file {} at line 1",
                    &citekey, &filepath
                );
                line_count = 0;
                break;
            }
        }

        // Exit TUI to enter editor
        tui.exit()?;
        // Use VISUAL or EDITOR. Set "vi" as last fallback
        let mut cmd: Command = EditorBuilder::new()
            .environment()
            .source(Some("vi"))
            .build()
            .unwrap();
        // Prepare arguments to open file at specific line
        let args: Vec<String> = vec![format!("+{}", line_count), filepath];
        let status = cmd.args(&args).status()?;
        if !status.success() {
            eprintln!("Spawning editor failed with status {}", status);
        }

        // Enter TUI again
        tui.enter()?;
        tui.terminal.clear()?;

        // Update the database and the lists to show changes
        self.update_lists();

        // Search for entry, selected before editing, by matching citekeys
        // Use earlier saved copy of citekey to match
        let mut idx_count = 0;
        loop {
            if self.entry_table.entry_table_items[idx_count]
                .citekey
                .contains(&saved_key)
            {
                break;
            }
            idx_count = idx_count + 1
        }

        // Set selected entry to vec-index of match
        self.entry_table.entry_table_state.select(Some(idx_count));

        Ok(())
    }

    // Search entry list
    pub fn search_entries(&mut self) {
        // Use snapshot of entry list saved when starting the search
        // so deleting a char, will show former entries too
        let orig_list = self.entry_table.entry_table_at_search_start.clone();
        let filtered_list =
            BibiSearch::search_entry_list(&mut self.search_struct.search_string, orig_list.clone());
        self.entry_table.entry_table_items = filtered_list;
        self.entry_table.sort_entry_table(false);
        self.entry_table.entry_scroll_state = ScrollbarState::content_length(
            self.entry_table.entry_scroll_state,
            self.entry_table.entry_table_items.len(),
        );
    }

    // Open file connected with entry through 'file' or 'pdf' field
    pub fn open_connected_file(&mut self) -> Result<()> {
        let idx = self.entry_table.entry_table_state.selected().unwrap();
        let filepath = &self.entry_table.entry_table_items[idx].filepath.clone();

        // Build command to execute pdf-reader. 'xdg-open' is Linux standard
        let cmd = {
            match std::env::consts::OS {
                "linux" => String::from("xdg-open"),
                "macos" => String::from("open"),
                "windows" => String::from("start"),
                _ => panic!("Couldn't detect OS for setting correct opener"),
            }
        };

        // Pass filepath as argument, pipe stdout and stderr to /dev/null
        // to keep the TUI clean (where is it piped on Windows???)
        let _ = Command::new(&cmd)
            .arg(&filepath)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .wrap_err("Opening file not possible");

        Ok(())
    }

    pub fn open_doi_url(&mut self) -> Result<()> {
        let idx = self.entry_table.entry_table_state.selected().unwrap();
        let web_adress = self.entry_table.entry_table_items[idx].doi_url.clone();

        // Resolve strings using the resolving function of dx.doi.org, so the
        // terminal is not blocked by the resolving process
        let url = if web_adress.starts_with("10.") {
            let prefix = "https://doi.org/".to_string();
            prefix + &web_adress
        } else if web_adress.starts_with("www.") {
            let prefix = "https://".to_string();
            prefix + &web_adress
        } else {
            web_adress
        };

        // Build command to execute browser. 'xdg-open' is Linux standard
        let cmd = {
            match std::env::consts::OS {
                "linux" => String::from("xdg-open"),
                "macos" => String::from("open"),
                "windows" => String::from("start"),
                _ => panic!("Couldn't detect OS for setting correct opener"),
            }
        };

        // Pass filepath as argument, pipe stdout and stderr to /dev/null
        // to keep the TUI clean (where is it piped on Windows???)
        let _ = Command::new(&cmd)
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .wrap_err("Opening file not possible");

        Ok(())
    }
}

impl Bibiman {
    // Tag List commands

    // Movement
    pub fn select_next_tag(&mut self, keywords: u16) {
        self.tag_list.tag_list_state.scroll_down_by(keywords);
        self.tag_list.tag_scroll_state = self
            .tag_list
            .tag_scroll_state
            .position(self.tag_list.tag_list_state.selected().unwrap());
    }

    pub fn select_previous_tag(&mut self, keywords: u16) {
        self.tag_list.tag_list_state.scroll_up_by(keywords);
        self.tag_list.tag_scroll_state = self
            .tag_list
            .tag_scroll_state
            .position(self.tag_list.tag_list_state.selected().unwrap());
    }

    pub fn select_first_tag(&mut self) {
        self.tag_list.tag_list_state.select_first();
        self.tag_list.tag_scroll_state = self.tag_list.tag_scroll_state.position(0);
    }

    pub fn select_last_tag(&mut self) {
        self.tag_list.tag_list_state.select_last();
        self.tag_list.tag_scroll_state = self
            .tag_list
            .tag_scroll_state
            .position(self.tag_list.tag_list_items.len());
    }

    pub fn get_selected_tag(&self) -> &str {
        let idx = self.tag_list.tag_list_state.selected().unwrap();
        let keyword = &self.tag_list.tag_list_items[idx];
        // let keyword = &self.tag_list.tag_list_items[idx].keyword;
        keyword
    }

    pub fn search_tags(&mut self) {
        let orig_list = &self.tag_list.tag_list_at_search_start;
        let filtered_list =
            BibiSearch::search_tag_list(&self.search_struct.search_string, orig_list.clone());
        self.tag_list.tag_list_items = filtered_list;
        // Update scrollbar length after filtering list
        self.tag_list.tag_scroll_state = ScrollbarState::content_length(
            self.tag_list.tag_scroll_state,
            self.tag_list.tag_list_items.len(),
        );
    }

    pub fn filter_tags_by_entries(&mut self) {
        let mut filtered_keywords: Vec<String> = Vec::new();

        let orig_list = &self.entry_table.entry_table_items;

        for e in orig_list {
            if !e.keywords.is_empty() {
                let mut key_vec: Vec<String> = e
                    .keywords
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                filtered_keywords.append(&mut key_vec);
            }
        }

        filtered_keywords.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        filtered_keywords.dedup();

        self.search_struct.filtered_tag_list = filtered_keywords.clone();
        self.tag_list.tag_list_items = filtered_keywords;
        self.tag_list.tag_scroll_state = ScrollbarState::content_length(
            self.tag_list.tag_scroll_state,
            self.tag_list.tag_list_items.len(),
        );
    }

    // Filter the entry list by tags when hitting enter
    // If already inside a filtered tag or entry list, apply the filtering
    // to the already filtered list only
    pub fn filter_for_tags(&mut self) {
        let orig_list = &self.entry_table.entry_table_items;
        let keyword = self.get_selected_tag();
        let filtered_list = BibiSearch::filter_entries_by_tag(&keyword, &orig_list);
        // self.tag_list.selected_keyword = keyword.to_string();
        self.tag_list.selected_keywords.push(keyword.to_string());
        self.entry_table.entry_table_items = filtered_list;
        // Update scrollbar state with new lenght of itemlist
        self.entry_table.entry_scroll_state = ScrollbarState::content_length(
            self.entry_table.entry_scroll_state,
            self.entry_table.entry_table_items.len(),
        );
        self.filter_tags_by_entries();
        self.toggle_area();
        self.entry_table.entry_table_state.select(Some(0));
        self.former_area = Some(FormerArea::TagArea);
    }
}

impl Bibiman {
    // Search Area

    // Enter the search area
    pub fn enter_search_area(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            if let Some(FormerArea::TagArea) = self.former_area {
                self.search_struct.inner_search = true
            }
            self.entry_table.entry_table_at_search_start =
                self.entry_table.entry_table_items.clone();
            self.former_area = Some(FormerArea::EntryArea)
        } else if let CurrentArea::TagArea = self.current_area {
            self.tag_list.tag_list_at_search_start = self.tag_list.tag_list_items.clone();
            self.former_area = Some(FormerArea::TagArea)
        }
        self.current_area = CurrentArea::SearchArea
    }

    // Confirm search: Search former list by pattern
    pub fn confirm_search(&mut self) {
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.current_area = CurrentArea::EntryArea;
            self.entry_table.entry_table_state.select(Some(0));
            self.entry_table.entry_table_at_search_start.clear();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.current_area = CurrentArea::TagArea;
            self.tag_list.tag_list_state.select(Some(0));
            self.tag_list.tag_list_at_search_start.clear();
        }
        self.former_area = Some(FormerArea::SearchArea);
        self.search_struct.search_string.clear();
    }

    // Break search: leave search area without filtering list
    pub fn break_search(&mut self) {
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.current_area = CurrentArea::EntryArea;
            self.entry_table.entry_table_state.select(Some(0));
            self.entry_table.entry_table_at_search_start.clear();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.current_area = CurrentArea::TagArea;
            self.tag_list.tag_list_state.select(Some(0));
            self.tag_list.tag_list_at_search_start.clear();
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

    pub fn search_list_by_pattern(&mut self, searchpattern: &Input) {
        self.search_struct.search_string = searchpattern.value().to_string();
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.search_entries();
            self.filter_tags_by_entries();
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.search_tags();
        }
    }
}
