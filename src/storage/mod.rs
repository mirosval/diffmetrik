use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct TimeTagged<T> {
    time: std::time::Duration,
    payload: T,
}

#[derive(Debug)]
pub struct StorageError {
    message: String,
}

impl std::convert::From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> StorageError {
        dbg!(&e);
        eprintln!("{:?}", &e);
        StorageError {
            message: "A storage error occurred".to_string(),
        }
    }
}

impl std::convert::From<serde_json::error::Error> for StorageError {
    fn from(e: serde_json::error::Error) -> StorageError {
        dbg!(&e);
        eprintln!("{:?}", &e);
        StorageError {
            message: "A storage error occurred".to_string(),
        }
    }
}

impl std::convert::From<std::time::SystemTimeError> for StorageError {
    fn from(e: std::time::SystemTimeError) -> StorageError {
        dbg!(&e);
        eprintln!("{:?}", &e);
        StorageError {
            message: "A storage error occurred".to_string(),
        }
    }
}

#[derive(Debug)]
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

    pub fn reset(&self) -> Result<(), StorageError> {
        File::create(&self.path)
            .map(|_| ())
            .map_err(|e| StorageError::from(e))
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
        dbg!(&buf);
        serde_json::from_str::<TimeTagged<T>>(&buf)
            .map_err(|e| StorageError::from(e))
            .map(|t| t.payload)
    }

    pub fn write<T>(&self, data: &T) -> Result<(), StorageError>
    where
        for<'de> T: Deserialize<'de>,
        T: Serialize,
    {
        // Open
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.path)?;
        // Lock
        file.lock_exclusive()?;
        // Read current value
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut reader = BufReader::new(&file);
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        // Get old timestamp
        let timetagged: Result<TimeTagged<T>, serde_json::error::Error> =
            serde_json::from_str(&buf);
        let should_write = timetagged
            .map(|t| {
                let sec = std::time::Duration::new(2, 0);
                let now = std::time::SystemTime::now();
                let dur: Result<std::time::Duration, std::time::SystemTimeError> =
                    now.duration_since(std::time::UNIX_EPOCH);
                let dur: std::time::Duration = dur.unwrap();
                let res = (dur - t.time) >= sec;
                dbg!(&res);
                res
            })
            .unwrap_or(true);
        if should_write {
            dbg!("writing");
            file.seek(std::io::SeekFrom::Start(0))?;
            let timetagged = TimeTagged {
                time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?,
                payload: data,
            };
            let serialized = serde_json::to_string(&timetagged).unwrap();
            file.write_all(serialized.as_bytes())?;
        }
        // Unlock
        file.unlock()?;

        Ok(())
    }
}
