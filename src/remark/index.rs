use crate::remark::Remark;
use crate::utils::data_structures::Map;
use std::path::{Path, PathBuf};

type RemarkId = u32;

pub struct RemarkIndex {
    files: Map<PathBuf, Vec<RemarkId>>,
    remarks: Vec<Remark>,
}

impl RemarkIndex {
    pub fn new(remarks: Vec<Remark>) -> Self {
        let mut files: Map<PathBuf, Vec<RemarkId>> = Map::default();
        for (index, remark) in remarks.iter().enumerate() {
            if let Some(ref location) = remark.function.location {
                files
                    .entry(Path::new(&location.file).to_path_buf())
                    .or_default()
                    .push(index as RemarkId);
            }
        }

        Self { files, remarks }
    }

    pub fn remarks(&self) -> &[Remark] {
        &self.remarks
    }

    pub fn files(&self) -> &Map<PathBuf, Vec<RemarkId>> {
        &self.files
    }
}
