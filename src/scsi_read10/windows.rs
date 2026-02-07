use super::ScsiDevice;
use anyhow::{Context, Result};
use std::ffi::c_void;
use std::fs::{self, File};
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::IscsiDisc::{
    IOCTL_SCSI_PASS_THROUGH_DIRECT, SCSI_IOCTL_DATA_IN, SCSI_PASS_THROUGH_DIRECT,
};
use windows::Win32::System::IO::DeviceIoControl;

pub struct WindowsScsiDevice {
    file: File,
}

impl WindowsScsiDevice {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let raw_path = path.as_ref().to_string_lossy();
        let device_path = normalize_device_path(&raw_path);
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&device_path)
            .with_context(|| format!("Failed to open device {device_path}"))?;
        Ok(Self { file })
    }
}

impl ScsiDevice for WindowsScsiDevice {
    fn read_cmd(&self, cdb: &[u8], data: &mut [u8]) -> Result<()> {
        let mut sptd: SCSI_PASS_THROUGH_DIRECT = unsafe { std::mem::zeroed() };

        sptd.Length = std::mem::size_of::<SCSI_PASS_THROUGH_DIRECT>() as u16;
        sptd.CdbLength = cdb.len() as u8;
        sptd.DataIn = SCSI_IOCTL_DATA_IN as u8;
        sptd.DataBuffer = data.as_mut_ptr() as *mut c_void;
        sptd.DataTransferLength = data.len() as u32;
        sptd.TimeOutValue = 5; // seconds

        if cdb.len() > 16 {
            return Err(anyhow::anyhow!("CDB too long for SCSI_PASS_THROUGH_DIRECT"));
        }
        sptd.Cdb[..cdb.len()].copy_from_slice(cdb);

        let mut bytes_returned: u32 = 0;
        let handle = HANDLE(self.file.as_raw_handle() as *mut c_void);

        let result = unsafe {
            DeviceIoControl(
                handle,
                IOCTL_SCSI_PASS_THROUGH_DIRECT,
                Some(&sptd as *const _ as *const c_void),
                std::mem::size_of::<SCSI_PASS_THROUGH_DIRECT>() as u32,
                Some(&mut sptd as *mut _ as *mut c_void),
                std::mem::size_of::<SCSI_PASS_THROUGH_DIRECT>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if result.is_err() {
            return Err(std::io::Error::last_os_error()).context("DeviceIoControl Failed");
        }

        if sptd.ScsiStatus != 0 {
            return Err(anyhow::anyhow!(
                "SCSI command failed with status: 0x{:02x}",
                sptd.ScsiStatus
            ));
        }

        Ok(())
    }
}

fn normalize_device_path(raw_path: &str) -> String {
    if raw_path.starts_with(r"\\.\") {
        return raw_path.to_string();
    }

    let bytes = raw_path.as_bytes();
    let is_drive = bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic();
    if is_drive {
        return format!(r"\\.\{}", &raw_path[..2]);
    }

    if raw_path.to_ascii_lowercase().starts_with("physicaldrive") {
        return format!(r"\\.\{raw_path}");
    }

    raw_path.to_string()
}
