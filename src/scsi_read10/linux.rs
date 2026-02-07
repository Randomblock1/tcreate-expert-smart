use anyhow::{anyhow, Result};
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::Path;

use super::ScsiDevice;

#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
mod sg {
    include!(concat!(env!("OUT_DIR"), "/sg_bindings.rs"));
}

// Create a typed ioctl wrapper
nix::ioctl_readwrite_bad!(sg_io_ioctl, sg::SG_IO, sg::sg_io_hdr);

pub struct LinuxScsiDevice {
    fd: OwnedFd,
}

impl LinuxScsiDevice {
    /// Opens a SCSI device for SG_IO access.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fd = open(path.as_ref(), OFlag::O_RDWR, Mode::empty())
            .map_err(|e| anyhow!("Failed to open {}: {}", path.as_ref().display(), e))?;
        Ok(Self { fd })
    }
}

impl ScsiDevice for LinuxScsiDevice {
    fn read_cmd(&self, cdb: &[u8], data: &mut [u8]) -> Result<()> {
        let mut sense = [0u8; 32];

        let mut hdr = sg::sg_io_hdr::default();
        hdr.interface_id = b'S' as _;
        hdr.dxfer_direction = sg::SG_DXFER_FROM_DEV;
        hdr.cmd_len = cdb.len() as _;
        hdr.mx_sb_len = sense.len() as _;
        hdr.dxfer_len = data.len() as _;
        hdr.dxferp = data.as_mut_ptr() as _;
        hdr.cmdp = cdb.as_ptr() as _;
        hdr.sbp = sense.as_mut_ptr() as _;
        hdr.timeout = 2000;

        unsafe {
            sg_io_ioctl(self.fd.as_raw_fd(), &mut hdr)
                .map_err(|e| anyhow!("SG_IO ioctl failed: {}", e))?;
        }

        if hdr.status != 0 || hdr.host_status != 0 || hdr.driver_status != 0 {
            return Err(anyhow!(
                "SCSI command failed (status=0x{:02x}, host=0x{:04x}, driver=0x{:04x}, sense={:x?})",
                hdr.status,
                hdr.host_status,
                hdr.driver_status,
                &sense[..hdr.sb_len_wr as usize]
            ));
        }

        Ok(())
    }
}
