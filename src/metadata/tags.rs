//! Utility functions for parsing tags correctly.

/// Sanitize a tag by removing leading and trailing whitespace and null characters.
pub fn sanitize_tag(tag: &str) -> String {
    tag.trim().replace('\0', "").to_string()
}
