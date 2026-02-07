use crate::read10::parse_read10;
use anyhow::{Result, anyhow};
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::uio::pread;
use std::os::fd::OwnedFd;
use std::path::Path;

pub fn open_readonly(path: &Path) -> Result<OwnedFd> {
    open(path, OFlag::O_RDONLY, Mode::empty())
        .map_err(|e| anyhow!("Failed to open {}: {}", path.display(), e))
}

pub fn read_read10(fd: &OwnedFd, block_size: u64, cdb: &[u8], data: &mut [u8]) -> Result<()> {
    let parsed = parse_read10(cdb, block_size, data.len())?;
    let offset = parsed.lba * block_size;
    let expected = parsed.expected_bytes;

    let n = pread(fd, &mut data[..expected], offset as i64).map_err(|e| {
        anyhow!(
            "pread at LBA {} (offset 0x{:x}) failed: {}",
            parsed.lba,
            offset,
            e
        )
    })?;

    if n < expected {
        return Err(anyhow!(
            "Short read at LBA {}: got {} bytes, expected {}",
            parsed.lba,
            n,
            expected
        ));
    }

    Ok(())
}
