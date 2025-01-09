//! Utility functions for parsing tags correctly.

use std::collections::HashSet;

/// Configuration for the tag parser.
pub struct ParserConfig {
    delimiters: Vec<char>,
    sort: bool,
}

impl ParserConfig {
    pub fn new() -> Self {
        Self {
            delimiters: vec![';', ':', ',', '/', '\0'],
            sort: true,
        }
    }

    /// Set the delimiters to use for parsing tags.
    pub fn with_delimiters(mut self, delimiters: Vec<char>) -> Self {
        self.delimiters = delimiters;
        self
    }

    /// Set whether to sort the tags before returning them.
    pub fn sort(mut self, sort: bool) -> Self {
        self.sort = sort;
        self
    }
}

/// Parse a string of tags into a vector of tags.
///
/// # Arguments
/// 
/// * `input` - The string of tags to parse.
/// * `config` - The configuration for the parser.
///
/// # Returns
/// 
/// A vector of tags.
pub fn parse_tags(input: &str, config: &ParserConfig) -> Vec<String> {
    let tags: HashSet<String> = input
        .split(|c| config.delimiters.contains(&c))
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();

    let mut result: Vec<String> = tags.into_iter().collect();

    if config.sort {
        result.sort();
    }

    result
}

/// Sanitize a tag by removing leading and trailing whitespace and null characters.
pub fn sanitize_tag(tag: &str) -> String {
    tag.trim().replace('\0', "").to_string()
}
