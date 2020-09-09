// use serde_json::Result;
use rusqlite;
use rusqlite::Connection;
use std::convert::From;
use std::error::Error;
use std::fs;
use std::result::Result;

use crate::db;

// each frame in watson is a json array with indices correspondig to the following fields:
// HEADERS = ('start', 'stop', 'project', 'id', 'tags', 'updated_at')
//               0        1       2        3      4         5

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
    let tx = conn.transaction()?;

    let project_id = match db::get_project_by_name(&tx, &frame.project)? {
        None => db::create_project(&tx, &frame.project)?,
        Some(p) => p.id,
    };
    println!("project '{}' has id {}", &frame.project, project_id);
    tx.commit()?;
    Ok(())
}

pub fn import_watson_frames(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string("./watson/frames.json").unwrap();
    let frames: Vec<RawFrame> = serde_json::from_str(&data)?;

    frames
        .into_iter()
        .map(|raw| Frame::from(raw))
        .for_each(|frame| {
            import_watson_frame(conn, frame).unwrap();
        });

    Ok(())
}
