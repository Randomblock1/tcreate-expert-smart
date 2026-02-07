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

#### Linux

```bash
sudo tcreate-expert-smart /dev/sdX
sudo tcreate-expert-smart /dev/sdX --json
```

#### MacOS

```bash
sudo tcreate-expert-smart /dev/diskX
sudo tcreate-expert-smart /dev/rdiskX
sudo tcreate-expert-smart /dev/diskX --json
```

#### Windows

```cmd
tcreate-expert-smart X:
tcreate-expert-smart \\.\PhysicalDriveX
tcreate-expert-smart PhysicalDriveX
tcreate-expert-smart X: --json
```
