use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::SystemTimeError;
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub enum StorageError {
    IO {
        source: std::io::Error,
        path: String,
    },
    Serialization {
        source: serde_json::Error,
    },
    Time {
        source: SystemTimeError,
    },
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match &self {
            StorageError::IO { source, .. } => source.to_string(),
            StorageError::Serialization { source } => source.to_string(),
            StorageError::Time { source } => source.to_string(),
        };
        write!(f, "Storage Error: {}", s)
    }
}

impl From<std::time::SystemTimeError> for StorageError {
    fn from(e: std::time::SystemTimeError) -> StorageError {
        StorageError::Time { source: e }
    }
}

type Result<T, E = StorageError> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
struct TimeTagged<T> {
    time: Duration,
    payload: T,
}

#[derive(Debug)]
pub struct Storage {
    path: PathBuf,
    min_duration: Duration,
}

impl Storage {
    pub fn new(file_name: String, min_duration: Duration, debug: bool) -> Storage {
        let mut p = env::temp_dir();
        p.push(file_name);
        let path = p;
        if debug {
            eprintln!("Storing data in: {:?}", &path);
        }
        Storage { path, min_duration }
    }

    pub fn reset(&self) -> Result<()> {
        File::create(&self.path)
            .map(|_| ())
            .map_err(|e| StorageError::IO {
                source: e,
                path: format!("{:?}", &self.path),
            })
    }

    pub fn read<T>(&self) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
        T: std::fmt::Debug,
    {
        let path = &self.path;
        let file = File::open(path).map_err(|e| StorageError::IO {
            source: e,
            path: format!("{:?}", path),
        })?;
        file.lock_shared().map_err(|e| StorageError::IO {
            source: e,
            path: format!("{:?}", path),
        })?;

        let mut reader = BufReader::new(&file);
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|e| StorageError::IO {
                source: e,
                path: format!("{:?}", path),
            })?;

        file.unlock().map_err(|e| StorageError::IO {
            source: e,
            path: format!("{:?}", path),
        })?;

        //dbg!(&buf);
        serde_json::from_str::<TimeTagged<T>>(&buf)
            .map(|t| t.payload)
            .map_err(|e| StorageError::Serialization { source: e })
    }

    pub fn write<T>(&self, data: &T) -> Result<()>
    where
        for<'de> T: Deserialize<'de>,
        T: Serialize,
        T: std::fmt::Debug,
    {
        // Open
        let path = &self.path;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.path)
            .map_err(|e| StorageError::IO {
                source: e,
                path: format!("{:?}", path),
            })?;

        // Lock
        file.lock_exclusive().map_err(|e| StorageError::IO {
            source: e,
            path: format!("{:?}", path),
        })?;
        // Read current value
        file.rewind().map_err(|e| StorageError::IO {
            source: e,
            path: format!("{:?}", path),
        })?;

        let mut reader = BufReader::new(&file);
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|e| StorageError::IO {
                source: e,
                path: format!("{:?}", path),
            })?;

        // Get old timestamp
        let timetagged: Result<TimeTagged<T>, serde_json::error::Error> =
            serde_json::from_str(&buf);
        let now: Duration = SystemTime::now().duration_since(UNIX_EPOCH)?;

        let should_write = timetagged
            .map(|t| (now - t.time) >= self.min_duration)
            .unwrap_or(true);
        if should_write {
            file.rewind().map_err(|e| StorageError::IO {
                source: e,
                path: format!("{:?}", path),
            })?;

            let timetagged = TimeTagged {
                time: SystemTime::now().duration_since(UNIX_EPOCH)?,
                payload: data,
            };
            let serialized = serde_json::to_string(&timetagged).unwrap();
            file.set_len(serialized.len() as u64)
                .map_err(|e| StorageError::IO {
                    source: e,
                    path: format!("{:?}", path),
                })?;

            file.write_all(serialized.as_bytes())
                .map_err(|e| StorageError::IO {
                    source: e,
                    path: format!("{:?}", path),
                })?;
        }
        // Unlock
        file.unlock().map_err(|e| StorageError::IO {
            source: e,
            path: format!("{:?}", path),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        test_string: String,
    }

    #[test]
    fn path_set_correctly() {
        let path = "diffmetrik_test_path.json".to_string();
        let s = Storage::new(path.to_string(), Duration::new(0, 0), true);
        let full_path = s.path;
        assert_eq!(full_path.ends_with(path), true);
    }

    #[test]
    fn file_reset() {
        let path = "diffmetrik_test_reset.json".to_string();
        remove_file(&path);
        let s = Storage::new(path.to_string(), Duration::new(0, 0), true);
        let full_path = &s.path;
        assert!(
            !full_path.exists(),
            "the file should not exist at the beginning of test"
        );
        match s.reset() {
            Ok(_) => assert!(true, "file reset successfully"),
            Err(e) => assert!(false, "error resetting file: {}", e),
        }
        let metadata = std::fs::metadata(full_path).expect("should get metadata");
        assert!(metadata.is_file(), "should be a file");
        assert!(metadata.len() == 0, "should be empty");
    }

    #[test]
    fn file_write() {
        let path = "diffmetrik_test_write.json".to_string();
        remove_file(&path);
        let s = Storage::new(path.to_string(), Duration::new(0, 0), true);
        let full_path = &s.path;
        assert!(
            !full_path.exists(),
            "the file should not exist at the beginning of test"
        );
        let w = TestStruct {
            test_string: "something".to_string(),
        };
        let res = s.write(&w);
        match res {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "writing failed with {}", e),
        }
        assert!(
            full_path.exists(),
            "the file should exist at the end of test"
        )
    }

    #[test]
    fn file_write_len() {
        let path = "diffmetrik_test_write_len.json".to_string();
        remove_file(&path);
        let s = Storage::new(path.to_string(), Duration::new(0, 0), true);
        let full_path = &s.path;
        let w1 = TestStruct {
            test_string: "something".to_string(),
        };
        s.write(&w1).expect("written w1");
        let meta1 = std::fs::metadata(&full_path).expect("metadata");
        let w2 = TestStruct {
            test_string: "som".to_string(),
        };
        let len1 = meta1.len();
        s.write(&w2).expect("written w2");
        let meta2 = std::fs::metadata(&full_path).expect("metadata");
        let len2 = meta2.len();
        assert!(
            dbg!(len1 > len2, len1, len2).0,
            "file was not properly truncated during write"
        )
    }

    #[test]
    fn file_read() {
        let path = "diffmetrik_test_read.json".to_string();
        let s = Storage::new(path.to_string(), Duration::new(0, 0), true);
        let payload = "something".to_string();
        let w = TestStruct {
            test_string: payload.clone(),
        };
        s.write(&w).expect("file written");
        let res: Result<TestStruct, _> = s.read();
        match res {
            Ok(res) => assert!(
                res.test_string == payload,
                "payload after read was different from payload written"
            ),
            Err(e) => assert!(false, "error reading: {}", e),
        }
    }

    fn remove_file(path: &String) {
        let s = Storage::new(path.clone(), Duration::new(0, 0), true);
        let full_path = s.path;
        if full_path.exists() {
            std::fs::remove_file(full_path).expect("removed temp file");
        }
    }
}
