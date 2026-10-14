//! One open framebuffer device.
use crate::errors::{Error, Result};
use crate::sys::{self, FbInfo, FbMap};
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;

pub struct Display {
    _file: File,
    pub info: FbInfo,
    map: FbMap,
}

impl Display {
    pub fn open(path: &Path) -> Result<Display> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Display::from_file(file)
    }

    /// Wraps a framebuffer fd this process already owns (the greeter
    /// receives one from the daemon rather than opening the device path
    /// itself — see [`crate::sys::adopt_fd_as_file`]).
    pub fn from_fd(fd: std::os::fd::RawFd) -> Result<Display> {
        Display::from_file(sys::adopt_fd_as_file(fd))
    }

    fn from_file(file: File) -> Result<Display> {
        let info = sys::fb_info(file.as_raw_fd())?;
        if info.width == 0 || info.height == 0 || info.width > 16384 || info.height > 16384 {
            return Err(Error::display("implausible framebuffer geometry"));
        }
        if !(8..=32).contains(&info.bpp) {
            return Err(Error::display("unsupported framebuffer bit depth"));
        }
        // smem_len (the driver's own reported mapping size) can legitimately
        // be larger than line_length * height (padding, multiple buffers);
        // map whichever is bigger so a present() can never write past what
        // is actually mapped.
        let visible_len = info.line_length as usize * info.height as usize;
        let map_len = visible_len.max(info.smem_len as usize).max(1);
        let map = FbMap::new(file.as_raw_fd(), map_len)?;
        Ok(Display { _file: file, info, map })
    }

    pub fn width(&self) -> u32 {
        self.info.width
    }
    pub fn height(&self) -> u32 {
        self.info.height
    }

    /// Writes one already-rendered frame (see
    /// [`crate::greeter::renderer::Canvas::blit_to_fb`]) straight into the
    /// mapped device memory.
    pub fn present(&mut self, canvas: &crate::greeter::renderer::Canvas) -> Result<()> {
        let needed = self.info.line_length as usize * self.info.height as usize;
        let buf = self.map.as_mut_slice();
        if buf.len() < needed {
            return Err(Error::display("framebuffer mapping smaller than one frame"));
        }
        canvas.blit_to_fb(&mut buf[..needed], &self.info).map_err(Error::display)
    }
}
