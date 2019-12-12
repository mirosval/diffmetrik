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
        source: std::time::SystemTimeError,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize, Deserialize)]
struct TimeTagged<T> {
    time: std::time::Duration,
    payload: T,
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
        let sec = std::time::Duration::new(2, 0);
        let now: std::time::Duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context(Time {})?;
        let should_write = timetagged
            .map(|t| {
                let res = (now - t.time) >= sec;
                //dbg!(&res);
                res
            })
            .unwrap_or(true);
        if should_write {
            //dbg!("writing");
            file.seek(std::io::SeekFrom::Start(0))
                .context(IO { path })?;
            let timetagged = TimeTagged {
                time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .context(Time {})?,
                payload: data,
            };
            let serialized = serde_json::to_string(&timetagged).unwrap();
            file.write_all(serialized.as_bytes()).context(IO { path })?;
        }
        // Unlock
        file.unlock().context(IO { path })?;

        Ok(())
    }
}
