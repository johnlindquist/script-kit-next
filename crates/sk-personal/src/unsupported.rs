use super::{Error, Result};
use std::path::Path;
pub(crate) struct Dir;
impl Dir {
    pub fn root(_: &Path) -> Result<Self> {
        Err(Error::Unsupported)
    }
    pub fn identity(&self) -> Result<(u64, u64)> {
        Err(Error::Unsupported)
    }
    pub fn path(&self, _: &[&str], _: bool) -> Result<Option<Self>> {
        Err(Error::Unsupported)
    }
    pub fn lock(&self) -> Result<()> {
        Err(Error::Unsupported)
    }
    pub fn read(&self, _: &str) -> Result<Option<Vec<u8>>> {
        Err(Error::Unsupported)
    }
    pub fn put(&self, _: &str, _: &[u8], _: bool) -> Result<()> {
        Err(Error::Unsupported)
    }
}
