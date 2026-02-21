# T-Create Expert S.M.A.R.T. tool

`tcreate-expert-smart` is a cross-platform Rust utility to read health and firmware information from **TeamGroup T-Create Expert S.M.A.R.T. microSD cards**.

These cards use a proprietary knock sequence of SCSI Read commands to retrieve health data. Contrary to typical S.M.A.R.T. data, this information is not accessible through standard ATA or SD interfaces, which is why a custom tool is necessary.

The original tool only worked in Windows, so this project reimplements the logic with support for Linux and macOS as well.

## Usage

### Installation

```bash
cargo install tcreate-expert-smart
```

### Run

Obviously, you need to replace the device paths with the correct ones for your system.

**Note**: Requires elevated privileges on Linux and MacOS.

Supplying the `-j` or `--json` option will return JSON instead of pretty output for use in automations.

#### Linux

```bash
sudo tcreate-expert-smart /dev/sdX
```

#### MacOS

```bash
sudo tcreate-expert-smart /dev/diskX
sudo tcreate-expert-smart /dev/rdiskX
```

#### Windows

```cmd
tcreate-expert-smart X:
tcreate-expert-smart \\.\PhysicalDriveX
tcreate-expert-smart PhysicalDriveX
```
### Example Output

```text
============================================================
          TEAMGROUP T-CREATE S.M.A.R.T. SD REPORT
============================================================
                   Health Remaining: 99%
     Firmware Version: SA3309_V11.1016.7162-TG(HYV7N48R
============================================================
```

with `--json`:

```text
{"health":99,"firmware_version":"SA3309_V11.1016.7162-TG(HYV7N48R"}
```

### Reverse Engineering the Windows Program
How'd I figure out the functionallity of a proprietary Windows app? As you'd expect, I first tried putting the tool's EXE in Ghidra. It was pretty clear there was some sort of obfuscation on there, as there weren't nearly as many function detections as I expected, and a bunch of data that wasn't recognizable. Some further inspection revealed it's a .NET application protected by .NET Reactor, so I ran it through .NETReactorSlayer. I put that into dotPeek, and finally, most of the code was there. Of course, it didn't make much sense initially. I saw some interesting arrays and offset numbers, but not really knowing what to look for, I was a bit lost. I decided to do a WireShark capture of the USB connection between my computer (well, the VM) and a microSD reader, which revealed it did a bunch of reads at increasingly odd addresses. I had enough information to work this out on my own, but deciding I had other things to do, I gave the decompiled code and packet captures to Claude Opus 4.5 to see if it could figure it out and save me a few hours of work. Suprisingly, this worked on the first try; it generated a Python file which read the health data. I converted this into a cross-platform Rust program without too much effort, although there wasn't a library to do cross-platform SCSI READ(10) operations, so that was a hassle.
