use lofty::file::FileType;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Similar to [`FileType`] from [lofty](https://crates.io/crates/lofty), except with [`Serialize`] and [`Deserialize`] traits implemented.
#[non_exhaustive]
#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SongFileType {
    Aac,
    Aiff,
    Ape,
    Flac,
    Mpeg,
    Mp4,
    Mpc,
    Opus,
    Vorbis,
    Speex,
    Wav,
    WavPack,
    Unknown,
}

impl From<FileType> for SongFileType {
    fn from(value: FileType) -> Self {
        match value {
            FileType::Aac => SongFileType::Aac,
            FileType::Aiff => SongFileType::Aiff,
            FileType::Ape => SongFileType::Ape,
            FileType::Flac => SongFileType::Flac,
            FileType::Mpeg => SongFileType::Mpeg,
            FileType::Mp4 => SongFileType::Mp4,
            FileType::Mpc => SongFileType::Mpc,
            FileType::Opus => SongFileType::Opus,
            FileType::Vorbis => SongFileType::Vorbis,
            FileType::Speex => SongFileType::Speex,
            FileType::Wav => SongFileType::Wav,
            FileType::WavPack => SongFileType::WavPack,
            _ => SongFileType::Unknown,
        }
    }
}
