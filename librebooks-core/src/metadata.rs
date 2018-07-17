use std::path;
use std::process;
use std::time;

use serde_json;

use errors::Result;

mod inner {

    #[derive(Debug, Clone, Deserialize)]
    pub struct Tags {
        pub title: Option<String>,
        pub artist: Option<String>,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct Chapter {
        pub tags: Tags,
        pub start_time: String,
        pub end_time: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Format {
        pub tags: Tags,
        pub duration: String,
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct Metadata {
        pub chapters: Vec<Chapter>,
        pub format: Format,
    }

}
#[derive(Debug, Clone, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start: time::Duration,
    pub end: time::Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub path: path::PathBuf,
    pub title: String,
    pub artist: String,
    pub duration: time::Duration,
    pub chapters: Vec<Chapter>,
}

impl Metadata {
    pub fn from_file(path: &path::PathBuf) -> Result<Metadata> {
        let cmd = process::Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_chapters")
            .arg("-show_format")
            .arg(path.to_str().unwrap())
            .output()?;

        let metadata: inner::Metadata = serde_json::from_slice(&cmd.stdout)?;

        let mut chapters: Vec<Chapter> = vec![];
        for chapter in metadata.chapters {
            chapters.push(Chapter {
                title: chapter.tags.title.unwrap_or_default(),
                start: millis_to_time(chapter.start_time),
                end: millis_to_time(chapter.end_time),
            });
        }

        return Ok(Metadata {
            path: path.clone(),
            chapters: chapters,
            title: metadata.format.tags.title.unwrap_or_default(),
            artist: metadata.format.tags.artist.unwrap_or_default(),
            duration: millis_to_time(metadata.format.duration),
        });
    }
}

pub fn millis_to_time(millis: String) -> time::Duration {
    time::Duration::from_millis((millis.parse::<f32>().unwrap() * 1000.0) as u64)
}
