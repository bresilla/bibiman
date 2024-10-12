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
use crate::backend::search::BibiSearch;
use ratatui::widgets::{ListState, ScrollbarState};

#[derive(Debug)]
pub struct TagList {
    pub tag_list_items: Vec<TagListItem>,
    pub tag_list_state: ListState,
    pub tag_scroll_state: ScrollbarState,
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
        let tag_list_items: Vec<TagListItem> = iter
            .into_iter()
            .map(|info| TagListItem::new(&info))
            .collect();
        let tag_list_state = ListState::default(); // for preselection: .with_selected(Some(0));
        let tag_scroll_state = ScrollbarState::new(tag_list_items.len());
        Self {
            tag_list_items,
            tag_list_state,
            tag_scroll_state,
        }
    }
}

impl App {
    // Tag List commands

    // Movement
    pub fn select_next_tag(&mut self) {
        self.tag_list.tag_list_state.select_next();
        self.tag_list.tag_scroll_state = self
            .tag_list
            .tag_scroll_state
            .position(self.tag_list.tag_list_state.selected().unwrap());
    }

    pub fn select_previous_tag(&mut self) {
        self.tag_list.tag_list_state.select_previous();
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
        let keyword = &self.tag_list.tag_list_items[idx].keyword;
        keyword
    }

    pub fn search_tags(&mut self) {
        let orig_list = &self.main_biblio.keyword_list;
        let filtered_list =
            BibiSearch::search_tag_list(&self.search_struct.search_string, orig_list.clone());
        self.tag_list = TagList::from_iter(filtered_list)
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
        self.tag_list = TagList::from_iter(filtered_keywords);
    }

    // Filter the entry list by tags when hitting enter
    // If already inside a filtered tag or entry list, apply the filtering
    // to the already filtered list only
    pub fn filter_for_tags(&mut self) {
        let orig_list = &self.entry_table.entry_table_items;
        let keyword = self.get_selected_tag();
        let filtered_list = BibiSearch::filter_entries_by_tag(&keyword, &orig_list);
        self.entry_table.entry_table_items = filtered_list;
        self.filter_tags_by_entries();
        self.toggle_area();
        self.former_area = Some(FormerArea::TagArea);
    }
}
