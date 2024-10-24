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
use crate::bibiman::search::BibiSearch;
use crate::bibiman::{Bibiman, FormerArea};
use crate::tui::Tui;
use color_eyre::eyre::{Context, Ok, Result};
use core::panic;
use editor_command::EditorBuilder;
use ratatui::widgets::ScrollbarState;
use std::process::{Command, Stdio};

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
        self.entry_table.entry_table_state.select_last();
        self.entry_table.entry_scroll_state = self
            .entry_table
            .entry_scroll_state
            .position(self.entry_table.entry_table_items.len());
    }

    /// Select next (right) column of entry table
    pub fn select_next_column(&mut self) {
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
        if self.entry_table.entry_table_reversed_sort {
            self.entry_table.sort_entry_table(false);
        }
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
        let orig_list = &self.main_biblio.keyword_list;
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
