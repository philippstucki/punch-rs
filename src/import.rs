// use serde_json::Result;
use chrono::{DateTime, TimeZone, Utc};
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

type RawFrame = (i64, i64, String, String, Vec<String>, u64);
#[derive(Debug)]
struct Frame {
    start: DateTime<Utc>,
    stop: DateTime<Utc>,
    project: String,
    tags: Vec<String>,
}
impl From<RawFrame> for Frame {
    fn from(item: RawFrame) -> Self {
        Frame {
            start: Utc.timestamp(item.0, 0),
            stop: Utc.timestamp(item.1, 0),
            project: item.2,
            tags: item.4,
        }
    }
}

fn import_watson_frame(conn: &Connection, frame: Frame) -> rusqlite::Result<()> {
    let project_id = match db::project_get_by_name(&conn, &frame.project)? {
        None => db::project_create(&conn, &frame.project)?,
        Some(p) => p.id,
    };

    let timeslice_id = db::timeslice_create(
        &conn,
        db::Timeslice {
            id: None,
            project_id: project_id,
            started_on: frame.start,
            stopped_on: Some(frame.stop),
        },
    )?;

    for tag in frame.tags {
        let tag_id = match db::tag_get_id_by_name_and_project_id(conn, &tag, project_id)? {
            None => db::tag_create(
                conn,
                db::TagCreate {
                    title: tag,
                    project_id,
                },
            )?,
            Some(t) => t,
        };
        println!("tag id: {}", tag_id);
        db::tag_assign_to_timeslice(conn, tag_id, timeslice_id)?;
    }

    Ok(())
}

pub fn import_watson_frames(conn: &mut Connection, file_name: &str) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(file_name).unwrap();
    let frames: Vec<RawFrame> = serde_json::from_str(&data)?;
    let tx = conn.transaction()?;

    frames
        .clone()
        .into_iter()
        .map(|raw| Frame::from(raw))
        .for_each(|frame| {
            import_watson_frame(&tx, frame).unwrap();
        });

    tx.commit()?;
    println!("imported {} frames", frames.len());

    Ok(())
}
