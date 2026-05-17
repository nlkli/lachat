use crate::Result;
use crate::models::openai::Message;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub pid: u32,
    pub host: String,
    pub port: u16,
    pub models_dir: Option<String>,
}

impl State {
    pub fn new(pid: u32, host: String, port: u16, models_dir: Option<String>) -> Self {
        Self { pid, host, port, models_dir }
    }
}

pub type Chat = Vec<Message>;

pub struct Session {
    path: PathBuf,
    _lock: File,
}

impl Session {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        fs::create_dir_all(&path)?;

        let lock_path = path.join("lock");
        let lock = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)?;

        let ret = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };

        if ret != 0 {
            return Result::Err(Box::new(io::Error::last_os_error()));
        }

        fs::create_dir_all(path.join("chats"))?;

        Ok(Session { path, _lock: lock })
    }

    #[inline(always)]
    fn state_path(&self) -> PathBuf {
        self.path.join("session.json")
    }

    #[inline(always)]
    fn chats_dir(&self) -> PathBuf {
        self.path.join("chats")
    }

    fn chat_path(&self, chat_id: &str) -> PathBuf {
        assert!(!chat_id.contains('/') && !chat_id.contains(".."));
        self.chats_dir().join(format!("{chat_id}.json"))
    }

    pub fn read_state(&self) -> Result<Option<State>> {
        self.read(self.state_path())
    }

    pub fn read_chat(&self, chat_id: &str) -> Result<Option<Chat>> {
        self.read(self.chat_path(chat_id))
    }

    fn read<T: serde::de::DeserializeOwned>(&self, path: PathBuf) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let obj = serde_json::from_str(&content)?;

        Ok(Some(obj))
    }

    pub fn write_state(&self, state: &State) -> Result<()> {
        self.write(self.state_path(), &serde_json::to_string(state)?)?;
        Ok(())
    }

    pub fn write_chat(&self, chat_id: &str, chat: &[Message]) -> Result<()> {
        self.write(self.chat_path(chat_id), &serde_json::to_string(chat)?)?;
        Ok(())
    }

    fn write<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        Ok(())
    }

    pub fn clear_chat(&self, chat_id: &str) -> Result<()> {
        Ok(fs::remove_file(self.chat_path(chat_id))?)
    }

    pub fn clear_all_chat(&self) -> Result<()> {
        for c in self.chat_list()? {
            self.clear_chat(&c)?;
        }
        Ok(())
    }

    pub fn chat_list(&self) -> Result<Vec<String>> {
        let mut list = Vec::new();
        for f in fs::read_dir(self.chats_dir())? {
            let file_name = f?.file_name().into_string().unwrap(); // ?
            let chat_id = file_name.trim_end_matches(".json");
            if !chat_id.is_empty() {
                list.push(chat_id.into());
            }
        }
        Ok(list)
    }
}
