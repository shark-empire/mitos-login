use super::display::Display;
use crate::errors::Result;
use std::path::{Path, PathBuf};

pub struct DisplayManager {
    path: PathBuf,
}

impl DisplayManager {
    pub fn new(path: PathBuf) -> DisplayManager {
        DisplayManager { path }
    }

    pub fn open_primary(&self) -> Result<Display> {
        Display::open(&self.path)
    }

    pub fn device_path(&self) -> &Path {
        &self.path
    }
}
