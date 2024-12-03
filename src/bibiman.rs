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
use crate::tui::popup::{PopupArea, PopupKind};
use crate::tui::Tui;
use crate::{bibiman::entries::EntryTable, bibiman::keywords::TagList};
use arboard::Clipboard;
use color_eyre::eyre::{Ok, Result};
use doi2bib;
use editor_command::EditorBuilder;
use futures::executor::block_on;
use ratatui::widgets::ScrollbarState;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::result::Result::Ok as AOk;
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
    PopupArea,
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
    // pub main_bibfiles: Vec<PathBuf>,
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
    // active popup
    pub popup_area: PopupArea,
}

impl Bibiman {
    // Constructs a new instance of [`App`].
    pub fn new(args: &CLIArgs) -> Result<Self> {
        // let main_bibfiles = args.fileargs.clone();
        let main_biblio = BibiSetup::new(&args.files);
        let tag_list = TagList::new(main_biblio.keyword_list.clone());
        let search_struct = BibiSearch::default();
        let entry_table = EntryTable::new(&main_biblio.entry_list);
        let current_area = CurrentArea::EntryArea;
        Ok(Self {
            // main_bibfiles,
            main_biblio,
            tag_list,
            search_struct,
            entry_table,
            scroll_info: 0,
            current_area,
            former_area: None,
            popup_area: PopupArea::default(),
        })
    }

    pub fn show_help(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            self.former_area = Some(FormerArea::EntryArea);
        } else if let CurrentArea::TagArea = self.current_area {
            self.former_area = Some(FormerArea::TagArea);
        }
        self.popup_area.is_popup = true;
        self.current_area = CurrentArea::PopupArea;
        self.popup_area.popup_kind = Some(PopupKind::Help);
    }

    pub fn add_entry(&mut self) {
        if let CurrentArea::EntryArea = self.current_area {
            self.former_area = Some(FormerArea::EntryArea);
        } else if let CurrentArea::TagArea = self.current_area {
            self.former_area = Some(FormerArea::TagArea);
        }
        self.popup_area.is_popup = true;
        self.current_area = CurrentArea::PopupArea;
        self.popup_area.popup_kind = Some(PopupKind::AddEntry);
    }

    pub fn handle_new_entry_submission(&mut self, args: &CLIArgs) {
        let new_entry_title = self.popup_area.add_entry_input.trim();
        let doi2bib = doi2bib::Doi2Bib::new().unwrap();
        let new_entry_future = doi2bib.resolve_doi(new_entry_title);
        let new_entry = block_on(new_entry_future);

        if let AOk(entry) = new_entry {
            // TODO: Add error handling for failed insert
            if self.append_to_file(args, &entry.to_string()).is_err() {
                self.popup_area.popup_kind = Some(PopupKind::MessageError);
                self.popup_area.popup_message = "Failed to add new entry".to_string();
            }
        // TODO: Add error handling for failed DOI lookup
        } else {
            self.popup_area.popup_kind = Some(PopupKind::MessageError);
            self.popup_area.popup_message = "Failed to add new entry".to_string();
        }
    }

    pub fn close_popup(&mut self) {
        // Reset all popup fields to default values
        self.popup_area = PopupArea::default();

        // Go back to previously selected area
        if let Some(FormerArea::EntryArea) = self.former_area {
            self.current_area = CurrentArea::EntryArea
        } else if let Some(FormerArea::TagArea) = self.former_area {
            self.current_area = CurrentArea::TagArea
        }

        // Clear former_area field
        self.former_area = None;
    }

    pub fn update_lists(&mut self, args: &CLIArgs) {
        self.main_biblio = BibiSetup::new(&args.files);
        self.tag_list = TagList::new(self.main_biblio.keyword_list.clone());
        self.entry_table = EntryTable::new(&self.main_biblio.entry_list);
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
        self.entry_table = EntryTable::new(&self.main_biblio.entry_list);
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

    pub fn run_editor(&mut self, args: &CLIArgs, tui: &mut Tui) -> Result<()> {
        // get filecontent and citekey for calculating line number
        let citekey: &str = &self.entry_table.entry_table_items
            [self.entry_table.entry_table_state.selected().unwrap()]
        .citekey
        .clone();

        // Add curly brace as prefix and comma as suffix that only
        // main citekeys are matched, not other fields like crossref
        let citekey_pattern: String = format!("{{{},", citekey);

        // Check if multiple files were passed to bibiman and
        // return the correct file path
        let filepath = if args.files.len() == 1 {
            args.files.first().unwrap().as_os_str()
        } else {
            let mut idx = 0;
            for f in &args.files {
                if search::search_pattern_in_file(&citekey_pattern, &f).is_some() {
                    break;
                }
                idx += 1;
            }
            args.files[idx].as_os_str()
        };
        let filecontent = fs::read_to_string(&filepath).unwrap();

        // Search the line number to place the cursor at
        let mut line_count = 0;

        for line in filecontent.lines() {
            line_count += 1;
            // if reaching the citekey break the loop
            // if reaching end of lines without match, reset to 0
            if line.contains(&citekey_pattern) {
                break;
            } else if line_count == filecontent.len() {
                eprintln!(
                    "Citekey {} not found, opening file {} at line 1",
                    citekey,
                    filepath.to_string_lossy()
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
        let status = cmd.arg(format!("+{}", line_count)).arg(filepath).status()?;
        if !status.success() {
            eprintln!("Spawning editor failed with status {}", status);
        }

        // Enter TUI again
        tui.enter()?;
        tui.terminal.clear()?;

        // Update the database and the lists to show changes
        Self::update_lists(self, args);

        // Search for entry, selected before editing, by matching citekeys
        // Use earlier saved copy of citekey to match
        let mut idx_count = 0;
        loop {
            if self.entry_table.entry_table_items[idx_count]
                .citekey
                .contains(citekey)
            {
                break;
            }
            idx_count += 1
        }

        // Set selected entry to vec-index of match
        self.entry_table.entry_table_state.select(Some(idx_count));

        Ok(())
    }

    pub fn append_to_file(&mut self, args: &CLIArgs, content: &str) -> Result<()> {
        // Determine the file path to append to
        let file_path = args.files.first().unwrap();
        // Open the file in append mode
        let mut file = OpenOptions::new().append(true).open(file_path).unwrap();
        // Optionally, add a newline before the content
        file.write_all(b"\n")?;
        // Format the content
        let formatted_content = Self::format_bibtex_entry(content);
        // Write the formatted content to the file
        file.write_all(formatted_content.as_bytes())?;
        // Update the database and the lists to reflect the new content
        self.update_lists(args);
        Ok(())
    }

    /// Formats a raw BibTeX entry string for better readability.
    pub fn format_bibtex_entry(entry: &str) -> String {
        let mut formatted = String::new();
        // Find the position of the first '{'
        if let Some(start_brace_pos) = entry.find('{') {
            // Copy the preamble (e.g., '@article{')
            let preamble = &entry[..start_brace_pos + 1];
            formatted.push_str(preamble);
            formatted.push('\n'); // Add newline
                                  // Now get the content inside the braces
            let rest = &entry[start_brace_pos + 1..];
            // Remove the last '}' at the end
            let rest = rest.trim_end();
            let rest = if rest.ends_with('}') {
                &rest[..rest.len() - 1]
            } else {
                rest
            };
            // Now we need to split the rest by commas, but commas can be inside braces or quotes
            // We'll parse the fields properly
            let mut fields = Vec::new();
            let mut current_field = String::new();
            let mut brace_level = 0;
            let mut in_quotes = false;
            for c in rest.chars() {
                match c {
                    '{' if !in_quotes => {
                        brace_level += 1;
                        current_field.push(c);
                    }
                    '}' if !in_quotes => {
                        brace_level -= 1;
                        current_field.push(c);
                    }
                    '"' => {
                        in_quotes = !in_quotes;
                        current_field.push(c);
                    }
                    ',' if brace_level == 0 && !in_quotes => {
                        // Outside of braces and quotes, comma separates fields
                        fields.push(current_field.trim().to_string());
                        current_field.clear();
                    }
                    _ => {
                        current_field.push(c);
                    }
                }
            }
            // Add the last field
            if !current_field.trim().is_empty() {
                fields.push(current_field.trim().to_string());
            }

            // Now reconstruct the entry with proper indentation
            for (i, field) in fields.iter().enumerate() {
                formatted.push_str("    ");
                formatted.push_str(field);
                // Add a comma if it's not the last field
                if i < fields.len() - 1 {
                    formatted.push(',');
                }
                formatted.push('\n');
            }
            formatted.push('}'); // Close the entry
            formatted
        } else {
            // No opening brace found, return the entry as is
            entry.to_string()
        }
    }

    // Search entry list
    pub fn search_entries(&mut self) {
        // Use snapshot of entry list saved when starting the search
        // so deleting a char, will show former entries too
        let orig_list = self.entry_table.entry_table_at_search_start.clone();
        let filtered_list =
            BibiSearch::search_entry_list(&self.search_struct.search_string, orig_list.clone());
        self.entry_table.entry_table_items = filtered_list;
        self.entry_table.sort_entry_table(false);
        self.entry_table.entry_scroll_state = ScrollbarState::content_length(
            self.entry_table.entry_scroll_state,
            self.entry_table.entry_table_items.len(),
        );
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
        // self.tag_list.tag_list_state.select_last(); // Doesn't work properly after upgrade to ratatui v.0.29
        self.tag_list
            .tag_list_state
            .select(Some(self.tag_list.tag_list_items.len() - 1));
        self.tag_list.tag_scroll_state = self
            .tag_list
            .tag_scroll_state
            .position(self.tag_list.tag_list_items.len());
    }

    pub fn get_selected_tag(&self) -> &str {
        let idx = self.tag_list.tag_list_state.selected().unwrap();
        &self.tag_list.tag_list_items[idx]
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

        filtered_keywords.sort_by_key(|a| a.to_lowercase());
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
        let filtered_list = BibiSearch::filter_entries_by_tag(keyword, orig_list);
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

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn citekey_pattern() {
        let citekey = format!("{{{},", "a_key_2001");

        assert_eq!(citekey, "{a_key_2001,")
    }
}
