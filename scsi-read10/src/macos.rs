use super::ScsiDevice;
use crate::block_reader::{open_readonly, read_read10};
use anyhow::Result;
use std::os::fd::OwnedFd;
use std::path::Path;

pub struct MacScsiDevice {
    fd: OwnedFd,
    block_size: u64,
}

impl MacScsiDevice {
    pub fn open(bsd_name: &str) -> Result<Self> {
        // Strip /dev/ prefix if present, then derive the raw device path
        let name = bsd_name.strip_prefix("/dev/").unwrap_or(bsd_name);

        // Use the raw (character) device to bypass the buffer cache.
        // This ensures every read actually reaches the hardware.
        let raw_name = if name.starts_with('r') {
            name.to_string()
        } else {
            format!("r{name}")
        };
        let path = format!("/dev/{raw_name}");

        let fd = open_readonly(Path::new(path.as_str()))?;

        Ok(Self {
            fd,
            block_size: 512,
        })
    }
}

impl ScsiDevice for MacScsiDevice {
    fn read_cmd(&self, cdb: &[u8], data: &mut [u8]) -> Result<()> {
        read_read10(&self.fd, self.block_size, cdb, data)
    }
}
