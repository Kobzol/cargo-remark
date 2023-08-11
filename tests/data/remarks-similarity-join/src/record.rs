use crate::common::Map;
use byteorder::ReadBytesExt;
use fxhash::FxBuildHasher;
use std::io::BufReader;

use crate::util::ForceUnwrap;

pub type Element = u32;

pub type Record = Vec<Element>;

type Ordering = byteorder::LittleEndian;

#[derive(Debug)]
pub struct Relation {
    // value -> set of records that contain it
    pub index: Map<Element, Vec<u32>>,
    pub records: Vec<Record>,
    pub record_lengths: Vec<u16>,
}

pub fn load_relation(path: &str) -> anyhow::Result<Relation> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(8192, file);

    let mut index = Map::with_capacity_and_hasher(1024, FxBuildHasher::default());
    let mut records = Vec::with_capacity(1024);
    let mut record_lengths = Vec::with_capacity(1024);

    loop {
        let size = match reader.read_u32::<Ordering>() {
            Ok(size) => size,
            Err(_) => break,
        };

        let mut record = Vec::with_capacity(size as usize);
        let record_id = records.len() as u32;

        for _ in 0..size {
            let value = reader.read_u32::<Ordering>().unwrap_force();
            let record_set = index.entry(value).or_insert_with(|| Vec::with_capacity(32));
            // ignore duplicates
            if std::intrinsics::unlikely(record_set.last() == Some(&record_id)) {
                continue;
            }
            record_set.push(record_id);
            record.push(value);
        }
        record.sort_unstable();

        assert!(record.len() <= u16::MAX as usize);
        record_lengths.push(record.len() as u16);
        records.push(record);
    }

    Ok(Relation {
        index,
        records,
        record_lengths,
    })
}
