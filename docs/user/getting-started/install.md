---
title: Install Local Raster
description: Download Local Raster for Windows 10 or 11, macOS or Linux from GitHub.
order: 10
verified: 0.8.2
---

Take a build from the [latest release](https://github.com/Anon5T4R/Local-Raster/releases/latest) on GitHub. In the file names below, `<version>` stands for the version number, for example `0.8.2`.

## Windows

Local Raster needs Windows 10 or later. On Windows 7 and 8 the installer stops with "Local Raster needs Windows 10 or later."

1. Open the [latest release](https://github.com/Anon5T4R/Local-Raster/releases/latest).
2. Download `localraster-<version>-windows-x64.msi`. It fits most PCs (64-bit Intel and AMD).
3. Run the file and follow the installer.

To run without installing, download `localraster-<version>-windows-x64-portable.zip` instead and unzip it.

| Your PC                            | Installer                                | Portable                                          |
|------------------------------------|------------------------------------------|---------------------------------------------------|
| 64-bit Intel or AMD                | `localraster-<version>-windows-x64.msi`   | `localraster-<version>-windows-x64-portable.zip`   |
| Arm (Snapdragon and other Arm PCs) | `localraster-<version>-windows-arm64.msi` | `localraster-<version>-windows-arm64-portable.zip` |
| 32-bit                             | `localraster-<version>-windows-x86.msi`   | `localraster-<version>-windows-x86-portable.zip`   |

The installers and executables are code-signed.

## macOS

1. Download `localraster-<version>-macos-universal.dmg`. It works on Apple silicon and Intel Macs, and is signed and notarized by Apple.
2. Open it and drag Local Raster to Applications.

## Linux

Pick one file from the release. Use `x86_64` for Intel and AMD, or `aarch64` for Arm.

| Format              | File                                         | Notes                                                                              |
|---------------------|----------------------------------------------|------------------------------------------------------------------------------------|
| AppImage            | `localraster-<version>-linux-x86_64.AppImage` | Runs anywhere and needs no install.                                                |
| Debian and Ubuntu   | `localraster-<version>-linux-x86_64.deb`      |                                                                                    |
| Fedora and openSUSE | `localraster-<version>-linux-x86_64.rpm`      |                                                                                    |
| Flatpak             | `localraster-<version>-linux-x86_64.flatpak`  | Run `flatpak install --user <file>`, then `flatpak run com.localraster.app`. |
| Tarball             | `localraster-<version>-linux-x86_64.tar.gz`   | Unpack it anywhere.                                                                |

The first time you run the AppImage, it adds its icon and menu entry under `~/.local/share`. Set `LOCALRASTER_NO_DESKTOP_INTEGRATION=1` to skip that. For RISC-V, take the `riscv64` file: `localraster-<version>-linux-riscv64.tar.gz`, `.deb` or `.rpm`. The RISC-V tarball needs glibc 2.39 or newer.

On a Wayland session, files dropped on the Local Raster window do not open yet. Use **File › Open…** instead, or copy the image in your file manager and paste it with Ctrl+V. To drag and drop, start Local Raster under XWayland with `WAYLAND_DISPLAY= localraster`. Local Raster shows the right command for your install in a notice until you dismiss it.

## FreeBSD

FreeBSD 14 (x86_64) has a tarball, `localraster-<version>-freebsd-x86_64.tar.gz`, laid out like `/usr/local`.

## Check your download

`SHA256SUMS.txt` on the release lists a checksum for every file.

## Next step

Open an image: [Open and save files](../guides/open-and-save-files.md).
