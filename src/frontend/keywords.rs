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

use super::app::{App, FormerArea};
use super::entries::EntryTable;
use crate::backend::search::BibiSearch;
use ratatui::widgets::ListState;

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

impl App {
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
}