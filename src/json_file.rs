use std::{fs, io, path::PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct JsonFile<T> {
    path: PathBuf,
    data: T,
}

impl<T: Serialize + DeserializeOwned> JsonFile<T> {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let bytes = fs::read(&path)?;
        let data = serde_json::from_slice(&bytes)?;
        Ok(Self { path, data })
    }

    pub fn flush(&self) -> io::Result<()> {
        let tmpfile = NamedTempFile::new_in(self.path.parent().unwrap())?;
        serde_json::to_writer_pretty(&tmpfile, &self.data)?;
        tmpfile.persist(&self.path)?;
        Ok(())
    }

    pub fn as_data(&self) -> &T {
        &self.data
    }

    pub fn name(&self) -> &str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }
}
