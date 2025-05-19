# GPUsage

[![Build Status](https://github.com/rice8y/gpusage/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/rice8y/gpusage/actions/workflows/CI.yml?query=branch%3Amain)
[![Release](https://github.com/rice8y/gpusage/actions/workflows/release.yml/badge.svg)](https://github.com/rice8y/gpusage/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/rice8y/gpusage?sort=semver)](https://github.com/rice8y/gpusage/releases)

## Installation

Replace `vX.Y.Z` below with the version you’re installing (e.g. `v0.1.0`), or automate via CI.

### Download from Releases

Prebuilt binaries are on the [Releases](https://github.com/rice8y/gpusage/releases) page. Choose the one for your OS/Arch and put it in your `PATH`.
<!--  -->
<!-- #### Linux -->

```bash
# Example for Linux
VERSION=vX.Y.Z
ARCH=x86_64-unknown-linux-gnu
URL=https://github.com/rice8y/gpusage/releases/download/${VERSION}/gpusage-${VERSION}-${ARCH}
wget $URL -O gpusage
chmod +x gpusage

# system-wide
sudo mv gpusage /usr/local/bin/gpusage

# or local
mv gpusage ~/.local/bin/gpusage
export PATH="$HOME/.local/bin:$PATH"
```
<!-- 
#### macOS (Homebrew alternative)

```bash
VERSION=vX.Y.Z
ARCH=x86_64-apple-darwin
URL=https://github.com/rice8y/gpusage/releases/download/${VERSION}/gpusage-${VERSION}-${ARCH}
curl -LO $URL
chmod +x gpusage

# system-wide
sudo mv gpusage /usr/local/bin/gpusage
```

#### Windows (PowerShell)

```bash
$version = 'vX.Y.Z'
$arch = 'x86_64-pc-windows-msvc'
$url = "https://github.com/rice8y/gpusage/releases/download/$version/gpusage-$version-$arch.exe"
Invoke-WebRequest $url -OutFile gpusage.exe

# Add to PATH or move:
Move-Item gpusage.exe C:\Windows\System32\gpusage.exe
``` -->

### Build from Source

If you have Rust installed:

```bash
git clone https://github.com/rice8y/gpusage.git
cd gpusage
cargo build --release
```

The compiled binary will be located at:
```bash
target/release/gpusage
```

>[!NOTE]
>If you install under a custom directory (e.g. `~/.local/bin`), ensure it’s in your `PATH`:
>
>```bash
>export PATH="$HOME/.local/bin:$PATH"
>```

## Usage

```bash
gpusage [OPTIONS] --end-time <YYYY-MM-DD-HH:MM:SS> --interval <SECONDS>
```

### Options

- `-e`, `--end-time <YYYY-MM-DD-HH:MM:SS>` : End datetime for monitoring (local time)
- `-i`, `--interval <SECONDS>` : Sampling interval in seconds
- `-b`, `--backup-interval <SECONDS>` : Interval in seconds for writing backups [default: 300]
- `-m`, `--backup-mode <MODE>` : Backup mode: 'new' for timestamped files, 'overwrite' to reuse a single file [default: new]
- `-p`, `--prefix <PREFIX>` : Filename prefix for backups [default: gpu_usage_backup]
- `-v`, `--verbose` : Enable verbose snapshot output
- `-h`, `--help` : Print help
- `-V`, `--version` : Print version