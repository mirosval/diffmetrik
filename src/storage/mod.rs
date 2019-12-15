use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use snafu::ResultExt;
use snafu::Snafu;
use std;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::SystemTimeError;
use std::time::UNIX_EPOCH;

#[derive(Debug, Snafu)]
pub enum Error {
    IO {
        source: io::Error,
        path: PathBuf,
    },
    Json {
        source: serde_json::Error,
        string: String,
    },
    Time {
        source: SystemTimeError,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

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
            .context(IO { path: &self.path })
    }

    pub fn read<T>(&self) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let path = &self.path;
        let file = File::open(path).context(IO { path })?;
        file.lock_shared().context(IO { path })?;
        let mut reader = BufReader::new(&file);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).context(IO { path })?;
        file.unlock().context(IO { path })?;
        // dbg!(&buf);
        serde_json::from_str::<TimeTagged<T>>(&buf)
            .map(|t| t.payload)
            .context(Json { string: buf })
    }

    pub fn write<T>(&self, data: &T) -> Result<()>
    where
        for<'de> T: Deserialize<'de>,
        T: Serialize,
    {
        // Open
        let path = &self.path;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.path)
            .context(IO { path })?;
        // Lock
        file.lock_exclusive().context(IO { path })?;
        // Read current value
        file.seek(std::io::SeekFrom::Start(0))
            .context(IO { path })?;
        let mut reader = BufReader::new(&file);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).context(IO { path })?;
        // Get old timestamp
        let timetagged: Result<TimeTagged<T>, serde_json::error::Error> =
            serde_json::from_str(&buf);
        let now: Duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context(Time {})?;
        let should_write = timetagged
            .map(|t| {
                let res = (now - t.time) >= self.min_duration;
                //dbg!(&res);
                res
            })
            .unwrap_or(true);
        if should_write {
            file.seek(std::io::SeekFrom::Start(0))
                .context(IO { path })?;
            let timetagged = TimeTagged {
                time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .context(Time {})?,
                payload: data,
            };
            let serialized = serde_json::to_string(&timetagged).unwrap();
            file.set_len(serialized.len() as u64).context(IO { path })?;
            file.write_all(serialized.as_bytes()).context(IO { path })?;
        }
        // Unlock
        file.unlock().context(IO { path })?;

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
            Err(e) => assert!(false, format!("error resetting file: {}", e)),
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
            Err(e) => assert!(false, format!("writing failed with {}", e)),
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
            Err(e) => assert!(false, format!("error reading: {}", e)),
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
