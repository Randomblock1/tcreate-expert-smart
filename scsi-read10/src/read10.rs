use anyhow::{Result, anyhow};

#[derive(Clone, Copy)]
pub struct Read10Cdb {
    bytes: [u8; 10],
}

impl Read10Cdb {
    /// Create a READ(10) CDB initialized with the READ(10) opcode.
    pub fn new() -> Self {
        let mut bytes = [0u8; 10];
        bytes[0] = 0x28; // Operation Code: READ(10)
        Self { bytes }
    }

    /// Set legacy LUN (Logical Unit Number) bits for older transports.
    pub fn set_lun_legacy(&mut self, lun: u8) {
        // Byte 1, bits 5-7: legacy LUN. Bits 0-4 are RDPROTECT/DPO/FUA/FUA_NV.
        self.bytes[1] = (self.bytes[1] & 0x1f) | ((lun & 0x07) << 5);
    }

    /// Set RDPROTECT (Read Protection) to select protection information handling.
    pub fn set_rdprotect(&mut self, rdprotect: u8) {
        // Byte 1, bits 0-2: RDPROTECT (0-7).
        self.bytes[1] = (self.bytes[1] & 0xf8) | (rdprotect & 0x07);
    }

    /// Set DPO (Disable Page Out) hint to avoid cache pollution for this read.
    pub fn set_dpo(&mut self, enabled: bool) {
        // Byte 1, bit 4: DPO.
        if enabled {
            self.bytes[1] |= 0x10;
        } else {
            self.bytes[1] &= !0x10;
        }
    }

    /// Set FUA (Force Unit Access) to bypass volatile cache on the device.
    pub fn set_fua(&mut self, enabled: bool) {
        // Byte 1, bit 3: FUA.
        if enabled {
            self.bytes[1] |= 0x08;
        } else {
            self.bytes[1] &= !0x08;
        }
    }

    /// Set FUA_NV (Force Unit Access Non-Volatile) to prefer non-volatile cache.
    pub fn set_fua_nv(&mut self, enabled: bool) {
        // Byte 1, bit 1: FUA_NV.
        if enabled {
            self.bytes[1] |= 0x02;
        } else {
            self.bytes[1] &= !0x02;
        }
    }

    /// Set the Logical Block Address (LBA) to read from.
    pub fn set_lba(&mut self, lba: u32) {
        // Bytes 2-5: Logical Block Address (big-endian).
        self.bytes[2..6].copy_from_slice(&lba.to_be_bytes());
    }

    /// Set the transfer length in blocks (not bytes).
    pub fn set_transfer_length(&mut self, blocks: u16) {
        // Bytes 7-8: Transfer length in blocks (big-endian). Byte 6 is group number.
        self.bytes[7..9].copy_from_slice(&blocks.to_be_bytes());
    }

    /// Set the Group Number field used by some devices for command grouping.
    pub fn set_group_number(&mut self, group: u8) {
        // Byte 6, bits 0-5: Group number.
        self.bytes[6] = group & 0x3f;
    }

    /// Set the Control byte (e.g., task attribute, ACA, or vendor-specific bits).
    pub fn set_control(&mut self, control: u8) {
        // Byte 9: Control.
        self.bytes[9] = control;
    }

    /// Return the raw 10-byte CDB for issuing the READ(10) command.
    pub fn as_bytes(&self) -> &[u8; 10] {
        &self.bytes
    }
}

pub struct Read10 {
    pub lba: u64,
    pub transfer_blocks: u64,
    pub expected_bytes: usize,
}

pub fn parse_read10(cdb: &[u8], block_size: u64, data_len: usize) -> Result<Read10> {
    if cdb.len() < 10 || cdb[0] != 0x28 {
        return Err(anyhow!(
            "Unsupported SCSI command 0x{:02x} (only READ(10) supported)",
            cdb.first().copied().unwrap_or(0)
        ));
    }

    let lba = u32::from_be_bytes([cdb[2], cdb[3], cdb[4], cdb[5]]) as u64;
    let transfer_blocks = u16::from_be_bytes([cdb[7], cdb[8]]) as u64;

    let expected_bytes_u64 = transfer_blocks.saturating_mul(block_size);
    let expected_bytes = usize::try_from(expected_bytes_u64)
        .map_err(|_| anyhow!("Transfer size too large: {} bytes", expected_bytes_u64))?;

    if data_len < expected_bytes {
        return Err(anyhow!(
            "Buffer too small: need {} bytes but got {}",
            expected_bytes,
            data_len
        ));
    }

    Ok(Read10 {
        lba,
        transfer_blocks,
        expected_bytes,
    })
}
