mod scsi_read10;
mod tcreate_expert_smart;

use anyhow::{bail, Context, Result};
use clap::Parser;
use crate::scsi_read10::{PlatformScsiDevice, Read10Cdb, ScsiDevice};
use std::io::Cursor;
use tcreate_expert_smart::SmartData;

/// Tool to read health data from TeamGroup T-CREATE SMART Expert microSD cards
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the device (e.g., /dev/sdb, /dev/disk10, \\.\PhysicalDrive1, or D:)
    #[arg(value_name = "DEVICE")]
    device: String,

    /// Output JSON instead of the formatted report
    #[arg(long, short = 'j')]
    json: bool,
}

// LBA sequence to "unlock" the SMART data.
// The last LBA read should contain the actual SMART data.
const KNOCK_SEQUENCE_LBAS: &[u64] = &[256, 4096, 768, 2048, 1280, 1536, 21376, 29556];

fn main() -> Result<()> {
    let args = Args::parse();
    let device_path = &args.device;

    if !args.json {
        println!("Opening device: {device_path}");
    }
    let device = PlatformScsiDevice::open(device_path)
        .with_context(|| format!("Failed to open device at {device_path}"))?;

    let mut final_data: Option<Vec<u8>> = None;

    for &lba in KNOCK_SEQUENCE_LBAS {
        let mut cdb = Read10Cdb::new();
        cdb.set_lun_legacy(1);
        cdb.set_lba(u32::try_from(lba)?);
        cdb.set_transfer_length(1);

        let mut data_buffer = vec![0u8; 512];
        data_buffer.fill(0);

        match device.read_cmd(cdb.as_bytes(), &mut data_buffer) {
            Ok(()) => {
                if lba == *KNOCK_SEQUENCE_LBAS.last().unwrap() {
                    final_data = Some(data_buffer.clone());
                }
            }
            Err(e) => {
                eprintln!("[!] Warning sending knock to LBA {lba}: {e}");
            }
        }
    }

    if let Some(data) = final_data {
        let mut reader = Cursor::new(&data);
        let smart_data =
            SmartData::read(&mut reader).context("Failed to parse SMART data structure")?;

        if smart_data.is_valid() {
            if args.json {
                let json = serde_json::to_string(&smart_data)
                    .context("Failed to serialize SMART data to JSON")?;
                println!("{json}");
            } else {
                print!("{smart_data}");
            }
        } else {
            bail!(
                "Incorrect SMART signature magic {:x?}.",
                smart_data.signature
            );
        }
    } else {
        bail!("Device did not respond to sequence or sequence incomplete.");
    }

    Ok(())
}
