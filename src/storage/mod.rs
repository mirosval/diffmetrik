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
        let file = File::open(&self.path);
        match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader::<BufReader<File>, T>(reader).map_err(|e| StorageError {
                    message: e.description().to_string(),
                })
            }
            Err(e) => Err(StorageError {
                message: e.description().to_string(),
            }),
        }
    }

    pub fn write<T>(&self, data: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(&data).unwrap();
        let f = std::fs::File::create(&self.path);
        match f {
            Ok(mut f) => f
                .write_all(serialized.as_bytes())
                .map(|_| ())
                .map_err(|e| StorageError {
                    message: e.description().to_string(),
                }),
            Err(e) => Err(StorageError {
                message: e.description().to_string(),
            }),
        }
    }
}
