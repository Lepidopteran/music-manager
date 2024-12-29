use id3::Tag;

mod album;
mod cover_art;
mod song;
mod error;

pub use {album::*, cover_art::*, song::*};

pub fn get_tag(path: &str) -> Option<Tag> {
    let tag = Tag::read_from_path(path).ok()?;
    Some(tag)
}
