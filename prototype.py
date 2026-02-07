import os
import sys
import ctypes
import fcntl

# =============================================================================
# HARDWARE CONFIGURATION (Source of Truth: C# Decompilation & USB Captures)
# =============================================================================

# The sequence of LBAs that triggers the controller to swap to the SMART page.
KNOCK_SEQUENCE_LBAS = [256, 4096, 768, 2048, 1280, 1536, 21376, 29556]

# Block Addressing / Offsets
BLOCK_SIZE    = 512
SIG_OFFSET    = 162
SIG_VALUE     = b'\x09\x33'
HEALTH_OFFSET = 192
FW_OFFSET     = 224

# =============================================================================
# SCSI / LINUX KERNEL CONSTANTS
# =============================================================================

SCSI_READ_10      = 0x28
SCSI_LUN_1_MASK   = 0x20
SG_IO             = 0x2285
SG_DXFER_FROM_DEV = -3

class SgIoHdr(ctypes.Structure):
    _fields_ = [
        ("interface_id", ctypes.c_int),
        ("dxfer_direction", ctypes.c_int),
        ("cmd_len", ctypes.c_ubyte),
        ("mx_sb_len", ctypes.c_ubyte),
        ("iovec_count", ctypes.c_ushort),
        ("dxfer_len", ctypes.c_uint),
        ("dxferp", ctypes.c_void_p),
        ("cmdp", ctypes.c_void_p),
        ("sbp", ctypes.c_void_p),
        ("timeout", ctypes.c_uint),
        ("flags", ctypes.c_uint),
        ("pack_id", ctypes.c_int),
        ("usr_ptr", ctypes.c_void_p),
        ("status", ctypes.c_ubyte),
        ("masked_status", ctypes.c_ubyte),
        ("msg_status", ctypes.c_ubyte),
        ("sb_len_wr", ctypes.c_ubyte),
        ("host_status", ctypes.c_ushort),
        ("driver_status", ctypes.c_ushort),
        ("resid", ctypes.c_int),
        ("duration", ctypes.c_uint),
        ("info", ctypes.c_uint)
    ]

# =============================================================================
# CORE LOGIC
# =============================================================================

def get_sd_details(device_path):
    print(f"[*] Opening {device_path} (SCSI Generic Mode)")

    try:
        fd = os.open(device_path, os.O_RDWR)
    except Exception as e:
        print(f"[!] Error: {e}")
        return

    data_buffer = (ctypes.c_ubyte * BLOCK_SIZE)()
    sense_buffer = (ctypes.c_ubyte * 32)()
    
    result_data = None
    
    # Perform the knock sequence on LUN 1
    for lba in KNOCK_SEQUENCE_LBAS:
        cdb = (ctypes.c_ubyte * 10)(
            SCSI_READ_10,
            SCSI_LUN_1_MASK,
            (lba >> 24) & 0xFF, (lba >> 16) & 0xFF, (lba >> 8) & 0xFF, lba & 0xFF,
            0x00, 0x00, 0x01, 0x00
        )

        header = SgIoHdr(
            interface_id=ord('S'),
            dxfer_direction=SG_DXFER_FROM_DEV,
            cmd_len=ctypes.sizeof(cdb),
            mx_sb_len=ctypes.sizeof(sense_buffer),
            dxfer_len=BLOCK_SIZE,
            dxferp=ctypes.addressof(data_buffer),
            cmdp=ctypes.addressof(cdb),
            sbp=ctypes.addressof(sense_buffer),
            timeout=2000
        )

        if fcntl.ioctl(fd, SG_IO, header) != 0:
            continue

        # Capture data from the final sector in the sequence
        if lba == KNOCK_SEQUENCE_LBAS[-1]:
            result_data = bytes(data_buffer)

    os.close(fd)

    if result_data:
        # 1. Verify Signature
        sig = result_data[SIG_OFFSET : SIG_OFFSET + 2]
        if sig != SIG_VALUE:
            print(f"[!] Fail: Incorrect signature {sig.hex()}. SMART page locked.")
            return

        # 2. Extract Health
        health = result_data[HEALTH_OFFSET]

        # 3. Extract Firmware String (starting at Offset 224)
        # We read until the end of the block or a null/invalid character
        fw_raw = result_data[FW_OFFSET:]
        fw_str = ""
        for b in fw_raw:
            if 32 <= b <= 126: # Valid ASCII range
                fw_str += chr(b)
            else:
                break

        print("\n" + "="*50)
        print(f"{'TEAMGROUP SMART SD REPORT':^50}")
        print("="*50)
        print(f"  Health Remaining: {health}%")
        print(f"  Firmware Version: {fw_str}")
        print("="*50 + "\n")
    else:
        print("[!] Fail: Device did not respond to sequence.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: sudo python3 tg_info.py /dev/sgX")
        sys.exit(1)
        
    get_sd_details(sys.argv[1])