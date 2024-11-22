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

use std::ffi::{OsStr, OsString};

use crate::bibiman::bibisetup::BibiData;
use ratatui::widgets::{ScrollbarState, TableState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryTableColumn {
    Authors,
    Title,
    Year,
    Pubtype,
}

// Define list containing entries as table
#[derive(Debug, PartialEq, Eq)]
pub struct EntryTable {
    pub entry_table_items: Vec<EntryTableItem>,
    pub entry_table_at_search_start: Vec<EntryTableItem>,
    pub entry_table_selected_column: EntryTableColumn,
    pub entry_table_sorted_by_col: EntryTableColumn,
    pub entry_table_reversed_sort: bool,
    pub entry_table_state: TableState,
    pub entry_scroll_state: ScrollbarState,
    pub entry_info_scroll: u16,
    pub entry_info_scroll_state: ScrollbarState,
}

impl EntryTable {
    pub fn new(entry_list: &[BibiData]) -> Self {
        let entry_table_items = Self::set_entry_table(entry_list);
        let entry_table_state = TableState::default()
            .with_selected(0)
            .with_selected_column(0)
            .with_selected_cell(Some((0, 0)));
        let entry_scroll_state = ScrollbarState::new(entry_table_items.len());
        let entry_info_scroll_state = ScrollbarState::default();
        Self {
            entry_table_items,
            entry_table_at_search_start: Vec::new(),
            entry_table_selected_column: EntryTableColumn::Authors,
            entry_table_sorted_by_col: EntryTableColumn::Authors,
            entry_table_reversed_sort: false,
            entry_table_state,
            entry_scroll_state,
            entry_info_scroll: 0,
            entry_info_scroll_state,
        }
    }

    pub fn set_entry_table(entry_list: &[BibiData]) -> Vec<EntryTableItem> {
        let mut entry_table: Vec<EntryTableItem> = entry_list
            .iter()
            .map(|e| EntryTableItem {
                authors: e.authors.clone(),
                short_author: String::new(),
                title: e.title.clone(),
                year: e.year.clone(),
                pubtype: e.pubtype.clone(),
                keywords: e.keywords.clone(),
                citekey: e.citekey.clone(),
                abstract_text: e.abstract_text.clone(),
                doi_url: e.doi_url.clone(),
                filepath: e.filepath.clone(),
                subtitle: e.subtitle.clone(),
            })
            .collect();

        entry_table.sort_by(|a, b| a.authors.to_lowercase().cmp(&b.authors.to_lowercase()));
        entry_table
    }

    // Sort entry table by specific column.
    // Toggle sorting by hitting same key again
    pub fn sort_entry_table(&mut self, toggle: bool) {
        if toggle {
            self.entry_table_reversed_sort = !self.entry_table_reversed_sort;
        }
        if self.entry_table_selected_column != self.entry_table_sorted_by_col {
            self.entry_table_reversed_sort = false
        }
        self.entry_table_sorted_by_col = self.entry_table_selected_column.clone();
        if self.entry_table_reversed_sort {
            match self.entry_table_selected_column {
                EntryTableColumn::Authors => self
                    .entry_table_items
                    .sort_by(|a, b| b.authors.to_lowercase().cmp(&a.authors.to_lowercase())),
                EntryTableColumn::Title => self
                    .entry_table_items
                    .sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase())),
                EntryTableColumn::Year => self
                    .entry_table_items
                    .sort_by(|a, b| b.year.to_lowercase().cmp(&a.year.to_lowercase())),
                EntryTableColumn::Pubtype => self
                    .entry_table_items
                    .sort_by(|a, b| b.pubtype.to_lowercase().cmp(&a.pubtype.to_lowercase())),
            }
        } else if !self.entry_table_reversed_sort {
            match self.entry_table_selected_column {
                EntryTableColumn::Authors => self
                    .entry_table_items
                    .sort_by(|a, b| a.authors.to_lowercase().cmp(&b.authors.to_lowercase())),
                EntryTableColumn::Title => self
                    .entry_table_items
                    .sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
                EntryTableColumn::Year => self
                    .entry_table_items
                    .sort_by(|a, b| a.year.to_lowercase().cmp(&b.year.to_lowercase())),
                EntryTableColumn::Pubtype => self
                    .entry_table_items
                    .sort_by(|a, b| a.pubtype.to_lowercase().cmp(&b.pubtype.to_lowercase())),
            }
        }
    }
}

// Define contents of each entry table row
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryTableItem {
    pub authors: String,
    pub short_author: String,
    pub title: String,
    pub year: String,
    pub pubtype: String,
    pub keywords: String,
    pub citekey: String,
    pub abstract_text: String,
    pub doi_url: Option<String>,
    pub filepath: Option<OsString>,
    pub subtitle: Option<String>,
}

impl EntryTableItem {
    // This functions decides which fields are rendered in the entry table
    // Fields which should be usable but not visible can be left out
    pub fn ref_vec(&mut self) -> Vec<&str> {
        self.short_author = match self.authors.split_once(",") {
            Some((first, _rest)) => {
                if self.authors.contains("(ed.)") {
                    let first_author = format!("{} et al. (ed.)", first);
                    first_author
                } else {
                    let first_author = format!("{} et al.", first);
                    first_author
                }
            }
            None => String::from(""),
        };

        vec![
            {
                if self.short_author.is_empty() {
                    &self.authors
                } else {
                    &self.short_author
                }
            },
            &self.title,
            &self.year,
            &self.pubtype,
        ]
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

    pub fn doi_url(&self) -> &str {
        self.doi_url.as_ref().unwrap()
    }

    pub fn filepath(&self) -> &OsStr {
        self.filepath.as_ref().unwrap()
    }

    pub fn subtitle(&self) -> &str {
        self.subtitle.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::EntryTableItem;

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

    #[test]
    fn shorten_authors() {
        let mut entry: EntryTableItem = EntryTableItem {
            authors: "Miller, Schmitz, Bernard".to_string(),
            short_author: "".to_string(),
            title: "A title".to_string(),
            year: "2000".to_string(),
            pubtype: "article".to_string(),
            keywords: "key1, key2".to_string(),
            citekey: "miller_2000".to_string(),
            abstract_text: "An abstract".to_string(),
            doi_url: None,
            filepath: None,
            subtitle: None,
        };

        let entry_vec = EntryTableItem::ref_vec(&mut entry);

        let mut entry_editors: EntryTableItem = EntryTableItem {
            authors: "Miller, Schmitz, Bernard (ed.)".to_string(),
            short_author: "".to_string(),
            title: "A title".to_string(),
            year: "2000".to_string(),
            pubtype: "article".to_string(),
            keywords: "key1, key2".to_string(),
            citekey: "miller_2000".to_string(),
            abstract_text: "An abstract".to_string(),
            doi_url: None,
            filepath: None,
            subtitle: None,
        };

        let entry_vec_editors = EntryTableItem::ref_vec(&mut entry_editors);

        assert_eq!(
            entry_vec,
            vec!["Miller et al.", "A title", "2000", "article"]
        );
        assert_eq!(
            entry_vec_editors,
            vec!["Miller et al. (ed.)", "A title", "2000", "article"]
        )
    }
}
