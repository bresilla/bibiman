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

use super::entries::EntryTableItem;
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf};

#[derive(Debug, Default)]
pub struct BibiSearch {
    pub search_string: String, // Search string show in footer, used for search
    pub inner_search: bool,    // True, if we trigger a search for already filtered list
    pub filtered_tag_list: Vec<String>,
}

impl BibiSearch {
    // Stringify EntryTableItem by joining/concat
    fn convert_to_string(inner_vec: &EntryTableItem) -> String {
        format!(
            "{} {} {} {} {} {}",
            &inner_vec.authors,
            &inner_vec.title,
            &inner_vec.year,
            &inner_vec.pubtype,
            &inner_vec.keywords,
            &inner_vec.citekey
        )
    }

    // Return a filtered entry list
    pub fn search_entry_list(
        search_pattern: &str,
        orig_list: Vec<EntryTableItem>,
    ) -> Vec<EntryTableItem> {
        // Create a hashmap to connect stingified entry with entry vec
        let mut entry_string_hm: HashMap<String, EntryTableItem> = HashMap::new();

        // Convert all entries to string and insert them into the hashmap
        // next to the original inner Vec<String> of the entry list
        for entry in orig_list {
            entry_string_hm.insert(Self::convert_to_string(&entry), entry);
        }

        // Set up matcher (TODO: One time needed only, move to higher level)
        let mut matcher = Matcher::new(Config::DEFAULT);

        // Filter the stringified entries and collect them into a vec
        let filtered_matches: Vec<String> = {
            let matches =
                Pattern::parse(search_pattern, CaseMatching::Ignore, Normalization::Smart)
                    .match_list(entry_string_hm.keys(), &mut matcher);
            matches.into_iter().map(|f| f.0.to_string()).collect()
        };

        // Create filtered entry list and push the inner entry vec's to it
        // Use the filtered stringified hm-key as index
        let mut filtered_list: Vec<EntryTableItem> = Vec::new();
        for m in filtered_matches {
            filtered_list.push(entry_string_hm[&m].to_owned());
        }
        filtered_list.sort();
        filtered_list
    }

    pub fn search_tag_list(search_pattern: &str, orig_list: Vec<String>) -> Vec<String> {
        // Set up matcher (TODO: One time needed only)
        let mut matcher = Matcher::new(Config::DEFAULT);

        // Filter the list items by search pattern
        let filtered_matches: Vec<String> = {
            let matches =
                Pattern::parse(search_pattern, CaseMatching::Ignore, Normalization::Smart)
                    .match_list(orig_list, &mut matcher);
            matches.into_iter().map(|f| f.0.to_string()).collect()
        };
        filtered_matches
    }

    pub fn filter_entries_by_tag(
        keyword: &str,
        orig_list: &Vec<EntryTableItem>,
    ) -> Vec<EntryTableItem> {
        let mut filtered_list: Vec<EntryTableItem> = Vec::new();

        // Loop over the whole given entry table
        // Check if the selected keyword is present in the current entry
        // If present, push the entry to the filtered list
        for e in orig_list {
            if e.keywords.contains(keyword) {
                filtered_list.push(e.to_owned());
            }
        }

        filtered_list
    }
}

pub fn search_pattern_in_file<'a>(pattern: &str, file: &'a PathBuf) -> Option<&'a OsStr> {
    let content = fs::read_to_string(file).unwrap();

    if content.contains(pattern) {
        Some(file.as_os_str())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_join() {
        let bibvec: EntryTableItem = EntryTableItem {
            authors: "Author".to_string(),
            short_author: "".to_string(),
            title: "Title".to_string(),
            year: "1999".to_string(),
            pubtype: "article".to_string(),
            keywords: "hello, bye".to_string(),
            citekey: "author_1999".to_string(),
            abstract_text: "An abstract with multiple sentences. Here is the second".to_string(),
            doi_url: Some("https://www.bibiman.org".to_string()),
            filepath: Some("/home/file/path.pdf".to_string().into()),
            subtitle: None,
        };

        let joined_vec = BibiSearch::convert_to_string(&bibvec);

        assert_eq!(
            joined_vec,
            "Author Title 1999 article hello, bye author_1999"
        )
    }
}
