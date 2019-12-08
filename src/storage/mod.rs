use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug)]
pub struct StorageError {
    message: String,
}

impl std::convert::From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> StorageError {
        eprintln!("{:?}", &e);
        StorageError {
            message: "A storage error occurred".to_string(),
        }
    }
}

pub struct Storage {
    path: PathBuf,
}

impl Storage {
    pub fn new(debug: bool) -> Storage {
        let mut p = env::temp_dir();
        p.push("diffstat.json");
        let path = p;
        if debug {
            eprintln!("Storing data in: {:?}", &path);
        }
        Storage { path }
    }

    pub fn read<T>(&self) -> Result<T, StorageError>
    where
        for<'de> T: Deserialize<'de>,
    {
        let file = File::open(&self.path)?;
        file.lock_shared()?;
        let mut reader = BufReader::new(&file);
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        file.unlock()?;
        serde_json::from_str(&buf).map_err(|e| StorageError {
            message: e.description().to_string(),
        })
    }

    pub fn write<T>(&self, data: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = std::fs::File::create(&self.path)?;
        file.lock_exclusive()?;
        file.write_all(serialized.as_bytes())?;
        file.unlock()?;
        Ok(())
    }
}
