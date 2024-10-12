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

use super::app::App;
use super::tui::Tui;
use crate::backend::search::BibiSearch;
use color_eyre::eyre::{Context, Ok, Result};
use core::panic;
use editor_command::EditorBuilder;
use itertools::Itertools;
use ratatui::widgets::{ScrollbarState, TableState};
use std::process::{Command, Stdio};

// Define list containing entries as table
#[derive(Debug)]
pub struct EntryTable {
    pub entry_table_items: Vec<EntryTableItem>,
    pub entry_table_state: TableState,
    pub entry_scroll_state: ScrollbarState,
    pub entry_info_scroll: u16,
    pub entry_info_scroll_state: ScrollbarState,
}

impl FromIterator<Vec<String>> for EntryTable {
    fn from_iter<T: IntoIterator<Item = Vec<String>>>(iter: T) -> Self {
        let entry_table_items: Vec<EntryTableItem> = iter
            .into_iter()
            .sorted()
            // 0: authors, 1: title, 2: date, 3: pubtype, 4: keywords, 5: citekey
            // 6: abstract, 7: doi/url, 8: pdf filepath
            // See backend/bib.rs BibiEntry impl
            .map(|i| {
                EntryTableItem::new(
                    &i[0], &i[1], &i[2], &i[3], &i[4], &i[5], &i[6], &i[7], &i[8],
                )
            })
            .collect();
        let entry_table_state = TableState::default().with_selected(0);
        let entry_scroll_state = ScrollbarState::new(entry_table_items.len());
        let entry_info_scroll_state = ScrollbarState::default();
        Self {
            entry_table_items,
            entry_table_state,
            entry_scroll_state,
            entry_info_scroll: 0,
            entry_info_scroll_state,
        }
    }
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
    pub abstract_text: String,
    pub doi_url: String,
    pub filepath: String,
}

impl EntryTableItem {
    pub fn new(
        authors: &str,
        title: &str,
        year: &str,
        pubtype: &str,
        keywords: &str,
        citekey: &str,
        abstract_text: &str,
        doi_url: &str,
        filepath: &str,
    ) -> Self {
        Self {
            authors: authors.to_string(),
            title: title.to_string(),
            year: year.to_string(),
            pubtype: pubtype.to_string(),
            keywords: keywords.to_string(),
            citekey: citekey.to_string(),
            abstract_text: abstract_text.to_string(),
            doi_url: doi_url.to_string(),
            filepath: filepath.to_string(),
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

impl App {
    // Entry Table commands

    // Movement
    pub fn select_next_entry(&mut self) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        self.entry_table.entry_table_state.select_next();
        self.entry_table.entry_scroll_state = self
            .entry_table
            .entry_scroll_state
            .position(self.entry_table.entry_table_state.selected().unwrap());
    }

    pub fn select_previous_entry(&mut self) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        self.entry_table.entry_table_state.select_previous();
        self.entry_table.entry_scroll_state = self
            .entry_table
            .entry_scroll_state
            .position(self.entry_table.entry_table_state.selected().unwrap());
    }

    pub fn select_first_entry(&mut self) {
        self.entry_table.entry_info_scroll = 0;
        self.entry_table.entry_info_scroll_state =
            self.entry_table.entry_info_scroll_state.position(0);
        self.entry_table.entry_table_state.select_first();
        self.entry_table.entry_scroll_state = self.entry_table.entry_scroll_state.position(0);
    }

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
        let orig_list = {
            if self.search_struct.inner_search {
                let orig_list = &self.search_struct.filtered_entry_list_by_tags;
                orig_list
            } else {
                let orig_list = &self.biblio_data.entry_list.bibentries;
                orig_list
            }
        };
        let filtered_list =
            BibiSearch::search_entry_list(&mut self.search_struct.search_string, orig_list.clone());
        //search::search_entry_list(&self.search_string, orig_list.clone());
        self.search_struct.filtered_entry_list_by_search = filtered_list.clone();
        self.entry_table = EntryTable::from_iter(filtered_list);
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

#[cfg(test)]
mod tests {
    #[test]
    fn check_os() {
        let os = std::env::consts::OS;
        assert_eq!(
            os,
            "linux",
            "You're not coding on linux, but on {}... Switch to linux, now!",
            std::env::consts::OS
        )
    }
}
