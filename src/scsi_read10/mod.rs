#![allow(dead_code)]
use anyhow::Result;

#[cfg(unix)]
pub(crate) mod block_reader;
pub(crate) mod read10;
pub use read10::Read10Cdb;

pub trait ScsiDevice {
    /// Sends a SCSI READ command to the device.
    ///
    /// # Arguments
    /// * `cdb` - The Command Descriptor Block.
    /// * `data` - Buffer for data transfer. The size of this buffer determines the transfer length.
    fn read_cmd(&self, cdb: &[u8], data: &mut [u8]) -> Result<()>;
}

// Linux requires special handling to access SCSI devices via the SG_IO interface
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxScsiDevice as PlatformScsiDevice;

// Windows is obviously its own thing
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsScsiDevice as PlatformScsiDevice;

// macOS just needs to open the raw device
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacScsiDevice as PlatformScsiDevice;
