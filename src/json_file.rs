use std::{fs, io, path::PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonFile<T> {
    pub path: PathBuf,
    pub data: T,
}

impl<T: Serialize + DeserializeOwned> JsonFile<T> {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let bytes = fs::read(&path)?;
        let data = serde_json::from_slice(&bytes)?;
        Ok(Self { path, data })
    }

    /// Not atomic
    pub fn flush(&self) -> io::Result<()> {
        let data = serde_json::to_vec_pretty(&self.data)?;
        fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn name(&self) -> Option<&str> {
        Some(self.path.file_stem()?.to_str()?)
    }
}
