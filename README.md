# Binwalk v3

Binwalk is a fast, modern tool for analyzing binary blobs and firmware images for embedded file signatures, extraction, and entropy analysis. This project includes both a CLI and a web interface, with ready-to-use Docker images.

---

## 🚀 Quick Start (Docker CLI)

You can use Binwalk directly from Docker without installing any dependencies on your system. The official image is:

```
docker pull happybear21/binwalk
```

### Analyze a File

Mount your working directory (so Binwalk can access your files):

```
docker run --rm -v "$PWD:/analysis" happybear21/binwalk /analysis/your_firmware.bin
```

- Replace `your_firmware.bin` with your target file.
- All output/extractions will appear in your current directory.

### Show Help / CLI Options

```
docker run --rm happybear21/binwalk --help
```

### Extract Embedded Files

```
docker run --rm -v "$PWD:/analysis" happybear21/binwalk -e /analysis/your_firmware.bin
```

### Carving, Threads, and Advanced Options

- Carve unknown files:
  ```
  docker run --rm -v "$PWD:/analysis" happybear21/binwalk -c /analysis/your_firmware.bin
  ```
- Set number of threads:
  ```
  docker run --rm -v "$PWD:/analysis" happybear21/binwalk -t 4 /analysis/your_firmware.bin
  ```
- Use include/exclude filters:
  ```
  docker run --rm -v "$PWD:/analysis" happybear21/binwalk -y gzip -x zip /analysis/your_firmware.bin
  ```

### Recursively Extract (Matryoshka)

```
docker run --rm -v "$PWD:/analysis" happybear21/binwalk -M -e /analysis/your_firmware.bin
```

---


![binwalk v3](images/binwalk_animated.svg)

## What does it do?

Binwalk can identify, and optionally extract, files and data that have been embedded inside of other files.

While its primary focus is firmware analysis, it supports a [wide variety](https://github.com/ReFirmLabs/binwalk/wiki/Supported-Signatures) of file and data types.

Through [entropy analysis](https://github.com/ReFirmLabs/binwalk/wiki/Generating-Entropy-Graphs), it can even help to identify unknown compression or encryption!

Binwalk can be customized and [integrated](https://github.com/ReFirmLabs/binwalk/wiki/Using-the-Rust-Library) into your own Rust projects.

## How do I get it?

The easiest way to install Binwalk and all dependencies is to [build a Docker image](https://github.com/ReFirmLabs/binwalk/wiki/Building-A-Binwalk-Docker-Image).

Binwalk can also be [installed](https://github.com/ReFirmLabs/binwalk/wiki/Cargo-Installation) via the Rust package manager.

Or, you can [compile from source](https://github.com/ReFirmLabs/binwalk/wiki/Compile-From-Source)!

## How do I use it?

Usage is _**simple**_, analysis is _**fast**_, and results are _**detailed**_:

```
binwalk DIR-890L_AxFW110b07.bin
```
![example output](images/output.png)

Use `--help`, or check out the [Wiki](https://github.com/ReFirmLabs/binwalk/wiki#usage) for more advanced options!
