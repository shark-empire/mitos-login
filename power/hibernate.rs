//! Suspend-to-disk. Flagged as not implemented rather than guessed: it needs
//! a correctly sized and configured swap area (`resume=`/`resume_offset=` on
//! the kernel command line) that this daemon has no way to verify is set up
//! correctly, and a wrong guess here risks data loss — worse than simply not
//! offering the option.
use crate::errors::{Error, Result};

pub fn is_supported() -> bool {
    false
}

pub fn run() -> Result<()> {
    Err(Error::Unsupported("hibernate is not implemented yet"))
}
