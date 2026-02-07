use binrw::BinRead;
use serde::Serialize;
use std::fmt;

/// This module defines the structure of the data returned by the SD card, and provides parsing and display logic for it.
// #[binread]
#[derive(Debug, Serialize, BinRead)]
#[br(little)]
pub struct SmartData {
    // Signature is located 162 bytes into the data
    #[br(pad_before = 162)] // :162
    #[serde(skip_serializing)]
    pub signature: [u8; 2], // :164

    #[br(pad_before = 28)] // :192
    pub health: u8, // :193

    #[br(pad_before = 31)]
    // :224
    // I don't actually know how long the firmware version string is, but we keep reading until we hit non-ascii characters
    #[br(map = |bytes: Vec<u8>| String::from_utf8_lossy(&bytes).trim_matches(|c: char| !c.is_ascii_graphic()).to_string())]
    #[br(count = 32)] // Is probably less
    pub firmware_version: String,
}

impl SmartData {
    pub const EXPECTED_SIGNATURE_MAGIC: [u8; 2] = [0x09, 0x33];

    pub fn is_valid(&self) -> bool {
        self.signature == Self::EXPECTED_SIGNATURE_MAGIC
    }

    pub fn read<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self, binrw::Error> {
        Self::read_le(reader)
    }
}

impl fmt::Display for SmartData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const LINE_LENGTH: usize = 60;
        writeln!(f, "{}", "=".repeat(LINE_LENGTH))?;
        writeln!(f, "{:^60}", "TEAMGROUP T-CREATE S.M.A.R.T. SD REPORT")?;
        writeln!(f, "{}", "=".repeat(LINE_LENGTH))?;
        writeln!(f, "{:^60}", format!("Health Remaining: {}%", self.health))?;
        writeln!(
            f,
            "{:^60}",
            format!("Firmware Version: {}", self.firmware_version)
        )?;
        writeln!(f, "{}", "=".repeat(LINE_LENGTH))
    }
}
