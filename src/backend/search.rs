use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
use std::collections::HashMap;

use crate::frontend::entries::EntryTableItem;

#[derive(Debug)]
pub struct BibiSearch {
    pub search_string: String, // Search string show in footer, used for search
    pub inner_search: bool,    // True, if we trigger a search for already filtered list
    // pub entry_list_at_search_start: Vec<EntryTableItem>,
    pub filtered_entry_list_by_search: Vec<Vec<String>>, // Temporary holds entry list filtered by search pattern to refilter it
    pub filtered_entry_list_by_tags: Vec<Vec<String>>, // Holds entry list filtered by tag to filter it further by search
    pub filtered_tag_list: Vec<String>,
}

impl Default for BibiSearch {
    fn default() -> Self {
        Self {
            search_string: String::new(),
            inner_search: false,
            // entry_list_at_search_start: Vec::new(),
            filtered_entry_list_by_search: Vec::new(),
            filtered_entry_list_by_tags: Vec::new(),
            filtered_tag_list: Vec::new(),
        }
    }
}

impl BibiSearch {
    // Stringify EntryTableItem by joining/concat
    fn convert_to_string(inner_vec: &EntryTableItem) -> String {
        let entry_table_item_str = {
            format!(
                "{} {} {} {} {} {}",
                &inner_vec.authors,
                &inner_vec.title,
                &inner_vec.year,
                &inner_vec.pubtype,
                &inner_vec.keywords,
                &inner_vec.citekey
            )
        };
        entry_table_item_str
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_join() {
        let bibvec: EntryTableItem = EntryTableItem::new(
            "Author",
            "Title",
            "1999",
            "article",
            "hello, bye",
            "author_1999",
            "An abstract with multiple sentences. Here is the second",
            "https://www.bibiman.org",
            "/home/file/path.pdf",
        );

        let joined_vec = BibiSearch::convert_to_string(&bibvec);

        assert_eq!(
            joined_vec,
            "Author Title 1999 article hello, bye author_1999"
        )
    }
}
