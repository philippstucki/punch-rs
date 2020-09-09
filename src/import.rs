// use serde_json::Result;
use std::convert::From;
use std::error::Error;
use std::fs;
use std::result::Result;

use rusqlite;
use rusqlite::Connection;

use crate::db;

//               0        1       2        3      4         5
// HEADERS = ('start', 'stop', 'project', 'id', 'tags', 'updated_at')

type RawFrame = (u64, u64, String, String, Vec<String>, u64);
#[derive(Debug)]
struct Frame {
    start: u64,
    stop: u64,
    project: String,
    tags: Vec<String>,
}
impl From<RawFrame> for Frame {
    fn from(item: RawFrame) -> Self {
        Frame {
            start: item.0,
            stop: item.1,
            project: item.2,
            tags: item.4,
        }
    }
}

fn import_watson_frame(conn: &mut Connection, frame: Frame) -> rusqlite::Result<()> {
    let project = db::get_project_by_name(conn, frame.project);
    // println()
    Ok(())
}

pub fn import_watson_frames(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string("./frames.json").unwrap();
    let frames: Vec<RawFrame> = serde_json::from_str(&data)?;

    frames
        .into_iter()
        .map(|raw| Frame::from(raw))
        .for_each(|frame| {
            println!("{:?}", frame);
            import_watson_frame(conn, frame).unwrap();
        });

    Ok(())
}
