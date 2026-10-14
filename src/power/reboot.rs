use crate::errors::Result;
use crate::sys;

pub fn run() -> Result<()> {
    sys::reboot_now()
}
