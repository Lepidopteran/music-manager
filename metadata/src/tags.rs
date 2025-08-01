//! Contains tag types and utility functions.

/// Duplicate of [`TagType`](lofty::tag::TagType) from [lofty](https://crates.io/crates/lofty), except with [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize) traits implemented.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TagType {
    Ape,
    Id3v1,
    Id3v2,
    Mp4Ilst,
    VorbisComments,
    RiffInfo,
    AiffText,
}

impl From<TagType> for lofty::tag::TagType {
    fn from(value: TagType) -> Self {
        match value {
            TagType::Ape => lofty::tag::TagType::Ape,
            TagType::Id3v1 => lofty::tag::TagType::Id3v1,
            TagType::Id3v2 => lofty::tag::TagType::Id3v2,
            TagType::Mp4Ilst => lofty::tag::TagType::Mp4Ilst,
            TagType::VorbisComments => lofty::tag::TagType::VorbisComments,
            TagType::RiffInfo => lofty::tag::TagType::RiffInfo,
            TagType::AiffText => lofty::tag::TagType::AiffText,
        }
    }
}

impl From<lofty::tag::TagType> for TagType {
    fn from(value: lofty::tag::TagType) -> Self {
        match value {
            lofty::tag::TagType::Ape => TagType::Ape,
            lofty::tag::TagType::Id3v1 => TagType::Id3v1,
            lofty::tag::TagType::Id3v2 => TagType::Id3v2,
            lofty::tag::TagType::Mp4Ilst => TagType::Mp4Ilst,
            lofty::tag::TagType::VorbisComments => TagType::VorbisComments,
            lofty::tag::TagType::RiffInfo => TagType::RiffInfo,
            lofty::tag::TagType::AiffText => TagType::AiffText,

            // NOTE: Unsure why this is needed.
            _ => TagType::Id3v2,
        }
    }
}

/// Sanitize a tag by removing leading and trailing whitespace and null characters.
pub fn sanitize_tag(tag: &str) -> String {
    tag.trim().replace('\0', "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_tag() {
        assert_eq!(sanitize_tag("  hello  "), "hello");
        assert_eq!(sanitize_tag("hello\0"), "hello");
    }

    #[test]
    fn test_tag_type() {
        assert_eq!(TagType::from(lofty::tag::TagType::Id3v2), TagType::Id3v2);
    }
}
