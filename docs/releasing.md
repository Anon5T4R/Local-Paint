# Releasing LocalPaint

Every tag `vX.Y.Z` runs `.github/workflows/release.yml`. The workflow builds the Windows
installers and the Linux packages in a matrix-free pair of jobs, then publishes all of them in the
**GitHub Release** of that tag (`gh release create`, with generated notes). The pipeline is ours:
it calls no shared workflow of another organisation and needs no secret to run.

The other packagers — macOS, FreeBSD, the web build and Flatpak — stay in `packaging/` and are run
by hand when needed; the tag workflow doesn't touch them.

User-facing names say **LocalPaint**. Files, binaries and ids stay lowercase (`localpaint-<version>-<platform>-<arch>.<ext>`, `com.localpaint.app`).

## In-app update checks

The native desktop app checks the public GitHub latest-release endpoint in a background
thread once at launch, with no periodic checks or automatic retries. Preferences › General › Automatically
Check for Updates disables the startup check; Help › Check for Updates runs one manually.
Checks use a 15-second timeout and a 1 MiB response limit, need no GitHub token, and send
no document or personal data. Network failures are quiet during automatic checks and visible
in the manual check window. Concurrent requests share one check.
HTTPS validates certificates and host names against the operating system's trust store.

Tags must be semantic versions (`v0.8.0` or `0.8.0`); only published stable versions newer
than the installed workspace version trigger a notice. Prereleases do not trigger it.
Flatpak builds also check for releases; repository-managed installations are updated through
their package manager. Web/headless frontends have no update transport. No release-workflow
changes are required: publishing a newer stable GitHub Release makes it discoverable.

## Cutting a release

1. **Bump the version** on `main`. The only place it lives is `[workspace.package] version`
   in the root `Cargo.toml`:

   ```sh
   cargo xtask version                 # prints the current version, e.g. 0.19.0
   cargo xtask version set 0.20.0      # or 0.20.0-rc.1; updates Cargo.toml and Cargo.lock
   ```

   Commit the change (`Cargo.toml` + `Cargo.lock`) through the normal review flow, and merge it.
2. **Tag the commit** and push the tag: `git tag v0.20.0 && git push origin v0.20.0`. The workflow
   starts by itself.
3. **Wait for the release.** The Windows and Linux jobs build and package; the `publish` job then
   creates the GitHub Release for the tag with every artifact attached and the notes generated
   from the merged PRs.
4. **Check it.** Download an installer or two. If the release already exists (a re-run of the same
   tag), `gh release create` fails rather than replacing assets: delete the release first, or
   upload the new files with `gh release upload --clobber`.
5. **Add the scorecard deltas** to the release notes: what moved in
   [`docs/scorecard.md`](scorecard.md) since the previous release. Compare the summary table
   and the performance table of both tags (`git diff v<previous> v<this> -- docs/scorecard.md`)
   and list each area's done / partial / missing change, the scenarios that newly meet their
   budget or got slower, corpus floors raised, and the change in settings that do nothing.
   Quote numbers with the baseline's machine and load average.

A pre-release tag (`v0.20.0-rc.1`) publishes the same way; mark it as a pre-release in the GitHub
UI if it shouldn't be offered as "latest".

**Test runs:** *Actions → Release → Run workflow* runs the same jobs on a branch, without the
`publish` job (it only runs for `refs/tags/v*`). The artifacts are on the run's page.

## What gets built

| Platform | Artifacts | Built on |
|---|---|---|
| Windows 10+ x64 | `localpaint-<v>-windows-x64.msi`, `localpaint-<v>-windows-x64-portable.zip`, `localpaint-<v>-windows-x64-symbols.zip` | `windows-latest` |
| Linux x86_64 | `localpaint-<v>-linux-x86_64.{AppImage,AppImage.zsync,deb,rpm,tar.gz}`, `localpaint-cli-<v>-linux-x86_64.tar.gz` | `ubuntu-22.04` |

The Windows job runs `packaging/windows/package.ps1` (x64) and the Linux job
`packaging/linux/package.sh --formats "appimage deb rpm tar cli"`. The same scripts take `-Arch
x86|arm64` and `CROSS_ARCH`/`CROSS_TARGET` respectively, and the other packagers in `packaging/`
cover macOS, FreeBSD, Flatpak and the web build; none of those run for a tag.

Every binary reports its version, the commit and the build date: `localpaint --version`,
`localpaint-cli --version`, and *Help › About LocalPaint*. The packaging scripts fill
`LOCALPAINT_BUILD_SHA` and `LOCALPAINT_BUILD_DATE` when the CI or the caller leaves them empty
(`packaging/env.sh` for local builds), and `crates/engine/src/build_info.rs` reads them at compile
time. A plain `cargo build` doesn't set them and reports the workspace version as a dev build.

Both jobs build with the fonts vendored in this repository: the workflow sets
`CRAFT_FONTS_DIR=$GITHUB_WORKSPACE/assets/craft-fonts`, and the packaging scripts carry each font's
licence as `OFL-<family>.txt` (`copy_font_licences` in `packaging/env.sh`; the portable zip on
Windows; `Contents/Resources/Licenses` in the macOS app). A build without `CRAFT_FONTS_DIR` still
compiles and falls back to the system's fonts; the web build embeds none (see
`docs/development.md` › Fonts). Build option: `docs/development.md` › Fonts.

### macOS

`packaging/macos/package.sh` builds `aarch64-apple-darwin` and `x86_64-apple-darwin` with
`MACOSX_DEPLOYMENT_TARGET=11.0`, joins them with `lipo`, and assembles `LocalPaint.app`:

- `Info.plist` is generated from `Info.plist.in`. The bundle id is `com.localpaint.app`.
  The plist sets `LSMinimumSystemVersion` 11.0, `NSHighResolutionCapable`, and document types:
  `.pcraft` (Owner), plus PSD/PSB and the image formats LocalPaint reads (Alternate, so it never
  takes over Preview's defaults). The icon is `assets/app-icon/localpaint.icns`.
- **Signing** goes inside-out with the hardened runtime and a secure timestamp. The executable
  is signed first, then the bundle. There's no `--deep` on the final signature. The
  entitlements (`entitlements.plist`) are deliberately empty.
- **Notarization:** the app is zipped and sent with `xcrun notarytool submit --wait`, then the
  ticket is stapled to the app. The app goes on a DMG (`hdiutil`, with an `Applications` link
  to drag onto). The DMG is signed, notarized and stapled too. The script checks the results
  with `codesign --verify --strict`, `stapler validate` and `spctl -a -vvv`.
  Its Finder window (background, icon size and positions) comes from
  [`packaging/macos/dmg/`](../packaging/macos/dmg/README.md), and its volume is named `LocalPaint`
  without the version, which the window's background needs; the DMG file name keeps the version.
- **CLI:** `LocalPaint.app/Contents/MacOS/localpaint-cli` is bundled and signed so generated
  droplet scripts can run without installing the separate CLI download. The scripts prefer
  `LOCALPAINT_CLI` when set, then the bundled CLI in `/Applications` or `~/Applications`, then
  `localpaint-cli` on `PATH`. Finder drag-and-drop onto droplets still requires an app wrapper.
  The universal standalone `localpaint-cli` is signed with the same Developer ID, the hardened
  runtime and a secure timestamp (identifier `com.localpaint.app-cli`), zipped, and the zip
  is sent to `notarytool`. Only `.app`, `.dmg` and `.pkg` can hold a stapled ticket, not a bare
  Mach-O, so Gatekeeper looks the CLI's ticket up online the first time a downloaded
  (quarantined) copy runs. Offline, that first run can be refused until the Mac is online again.
- **Verification:** `packaging/macos/verify.sh` checks the artifacts as users download them. It
  unpacks the CLI zip and requires, for the binary inside, `codesign --verify --strict`, both
  architectures, the hardened runtime flag, a `Developer ID Application` authority from
  `APPLE_TEAM_ID`, a timestamp, and `spctl --assess --type install` reporting
  `source=Notarized Developer ID` (the same online lookup Gatekeeper does). For the DMG it runs
  `codesign --verify`, `stapler validate` and `spctl --type open`. Run it after packaging, as its
  own step. With signing secrets it fails on any miss; without
  them it only checks signature integrity and warns, like `package.sh`. Run it locally after a
  build: `packaging/macos/verify.sh --arch aarch64`.

Locally, without certificates, the script signs ad-hoc (`codesign -s -`) and skips notarization.
That's enough to check the bundle and the DMG on your own Mac:

```sh
packaging/macos/package.sh                    # universal; needs both rustup targets
packaging/macos/package.sh --arch aarch64     # quicker, host-only
open dist/release/localpaint-*-macos-*.dmg
```

### Windows

`packaging/windows/package.ps1 -Arch x64|x86` builds with `-C target-feature=+crt-static`.
The static C runtime means neither the MSI nor the portable zip needs the Visual C++
redistributable, which matters for a standalone installer and costs only about 100 KB. The flag
goes in `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`, so host build scripts aren't affected.

- `apps/localpaint/build.rs` and `apps/localpaint-cli/build.rs` embed the icon
  (`assets/app-icon/localpaint.ico`) and VERSIONINFO with the `winresource` crate
  (build dependency `0.1`). They only do this when targeting Windows. Elsewhere each
  script is a no-op, even if `LOCALPAINT_REQUIRE_WINRES` is set, and the web build
  doesn't touch that crate. Both set ProductName to LocalPaint, CompanyName to
  Learning Machines LLC and the same copyright line. The GUI description is
  "LocalPaint image editor" (`localpaint.exe`). The CLI description is
  "LocalPaint command-line interface", with OriginalFilename `localpaint-cli.exe`
  and InternalName `localpaint-cli`. Version strings are the Cargo package version,
  so a pre-release tag is kept; the numeric file and product versions are
  winresource's `major.minor.patch.0`. A missing resource compiler warns, so an
  optional cross-compile still links. `package.ps1` sets `LOCALPAINT_REQUIRE_WINRES=1`.
  The CLI build script then returns an error naming `rc.exe` or `windres` and does
  not panic.
- Release builds use the GUI subsystem, so Start Menu launches don't open a console window.
  The CLI stays console subsystem 3 so its output reaches the terminal.
- `localpaint.wxs` (WiX v5) is a **per-machine** package (`Scope="perMachine"`): it installs into
  `Program Files\LocalPaint` (`Program Files (x86)\LocalPaint` for the 32-bit build), needs an
  elevated setup, and refuses a per-user install (`<Launch Condition="ALLUSERS = 1 OR (Installed
  AND REMOVE = "ALL")">`, so an older per-user copy can still be uninstalled). It creates a plain
  Start Menu shortcut and a desktop shortcut — not advertised MSI shortcuts, whose descriptor
  Windows can't use as a file handler — each in its own component with an HKLM key path, so
  Windows Installer removes them on uninstall. LocalPaint becomes the default app for `.pcraft` and
  is listed under "Open with" for PSD/PSB and image files, it registers App Paths (Win+R
  `localpaint`), and it writes `HKLM\Software\Anon5T4R\LocalPaint\InstallDir` so the updater knows
  this is an MSI install and updates with the next MSI rather than the portable zip. The MSI
  version is the numeric `X.Y.Z`, because MSI has no pre-release field. Same-version upgrades are
  allowed so that release candidates replace each other; a downgrade is refused.
- Double-clicking the MSI opens the `WixUI_Minimal` wizard: welcome with the licence (the project's
  `LICENSE-MIT`/`LICENSE-APACHE` text), a progress page, and a Finish page that says where to find
  the app, with a ticked "Launch LocalPaint" box. The pages name the full version
  (`localpaint.en-us.wxl`). `package.ps1` draws the wizard's banner and side bitmaps from the app
  icon, so no WiX stock art ships. `msiexec /qn` installs silently per-machine, as the launch
  condition requires. Building needs the WiX UI
  and Util extensions: `wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`.
- Shortcut icon identifiers keep the executable's `.exe` extension: MSI uses the identifier
  as the cached icon filename, and an extensionless filename can render as a blank document
  icon. `packaging/windows/check-icons.ps1` checks the references and extensions; run it on a
  packaging change. `package.ps1` also validates the built MSI with ICE50 before signing it.
- **Portable zip:** it ships `packaging/windows/portable.txt` beside `localpaint.exe`. That
  marker (or a `LocalPaint.portable` file) switches on portable mode: preferences, presets,
  recovery autosaves and the GPU startup marker go to `LocalPaintData\` next to the exe instead
  of `%APPDATA%\LocalPaint`. If that folder isn't writable the app warns and uses `%APPDATA%`.
  The MSI has no marker. The logic is in `apps/localpaint/src/app_dirs.rs` and works the same
  on macOS and Linux.
- **Symbols zip:** `localpaint.pdb` and `localpaint_cli.pdb` from the same release build, plus
  `packaging/windows/symbols.txt` (how to load them in a profiler). An optional download, so the
  MSI and portable zip stay their size; it lets anyone profile the released exes with function
  names (#688). Only that build's PDBs match its exes, so they can't be rebuilt later; the
  release profile has no line tables, so they carry function names, not source lines.
- **Signing:** `packaging/windows/sign.ps1` signs both `.exe` files and then the `.msi` with
  `signtool`, using SHA-256 and an RFC 3161 timestamp. It uses whichever material is present:
  1. a `.pfx` file (`WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`), timestamped by
     DigiCert, or
  2. Azure Trusted Signing (`AZURE_*`): the script downloads the
     `Microsoft.Trusted.Signing.Client` dlib and timestamps with Microsoft's server. The package
     is pinned to a version and SHA-256 at the top of `sign.ps1` and refused on a mismatch, so an
     upgrade is a deliberate change to both.

  The script is the single place to change when the Windows signing setup changes.

Locally on Windows: `dotnet tool install -g wix --version 5.0.2`,
`wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2`, then
`pwsh packaging/windows/package.ps1 -Arch x64`.

### Linux

`packaging/linux/package.sh` stages one FHS tree and makes every format from it. The tree
holds both binaries, `com.localpaint.app.desktop`, hicolor icons from 16 px to 512 px
plus a scalable SVG, AppStream metainfo, a shared-mime-info file for `.pcraft`, `.psb` and
`.qoi`, and `localpaint-cli`'s man pages and bash, zsh and fish completions. The
`man-and-completions` example, built with the binaries, writes those; a cross build runs it
through `EMULATOR` and leaves them out without one. The zsh completion is in
`/usr/share/zsh/site-functions`, which Fedora's and openSUSE's zsh read; the `.deb` also links it
from `/usr/share/zsh/vendor-completions`, which Debian's zsh reads instead.

Why these formats:

- **AppImage** runs on any distribution without installing anything. The
  download-and-go option and the fallback for distros the packages don't cover. It's built
  with the maintained `AppImage/appimagetool`, whose static runtime doesn't need libfuse2.
  The packaging script verifies pinned AppImageTool 1.9.1 and type2 runtime 20251108 binaries.
  For an offline build, pre-seed `$CARGO_TARGET_DIR/appimagetool-1.9.1-$ARCH.AppImage` and
  `$CARGO_TARGET_DIR/type2-runtime-20251108-$ARCH` with the matching official release assets;
  the script checks both SHA-256 digests and makes the cached AppImageTool executable after verification;
  the runtime is only read as data.
  An `APPIMAGETOOL` override must point to that same verified AppImageTool binary and already be
  executable (`chmod +x` it before packaging).
  Each AppImage embeds update information (`gh-releases-zsync|…|latest|…`, #349) and ships
  with a `.zsync` beside it, so AppImageUpdate, AppImageLauncher and similar tools can find
  the next release and download only the blocks that changed.
  Its `AppRun` (`packaging/linux/AppRun`) also integrates with the desktop on launch: on
  Wayland the dock icon comes from the compositor resolving the window's app_id against an
  *installed* `.desktop` file, which an AppImage alone never provides — without this every
  AppImage run shows the generic Wayland logo (#593). Best effort: it installs
  `~/.local/share/applications/com.localpaint.app.desktop` with `Exec` rewritten to
  the AppImage's path (`TryExec` dropped, the binary isn't on the host PATH) and the
  hicolor icons beside it, rewriting only when the content changed, and any failure leaves
  the launch untouched. `LOCALPAINT_NO_DESKTOP_INTEGRATION=1` opts out; to uninstall, `rm
  ~/.local/share/applications/com.localpaint.app.desktop` and `find
  ~/.local/share/icons/hicolor -name 'com.localpaint.app*' -delete`. The script is
  covered by `packaging/linux/apprun-test.sh`.
  A leading `--cli` launches the bundled `usr/bin/localpaint-cli`, without desktop
  integration: `./localpaint-<version>-linux-x86_64.AppImage --cli mcp` serves MCP on
  stdio. A symlink named `localpaint-cli` also selects that binary via the runtime's
  `ARGV0`. Arguments and process streams pass through unchanged; the entry point
  uses `exec` so signals and exit status reach the caller directly. CLI routing is
  covered by `packaging/linux/apprun-cli-test.sh`.
- **.deb** covers Debian, Ubuntu, Mint, Pop!_OS and elementary. **.rpm** covers Fedora, RHEL
  and its clones, and openSUSE. Both integrate with the menu, MIME and icon caches (the
  `postinst.sh` hook) and uninstall cleanly. Both are built by
  [nfpm](https://nfpm.goreleaser.com/): one maintained, dependency-free tool and one config
  (`nfpm.yaml`). It doesn't need cargo-deb and cargo-generate-rpm metadata in our crates, and
  it can build both formats on Ubuntu.
- **.tar.gz** is for people who manage their own `/opt` or `~/.local`.
- **Flatpak bundle** (`.flatpak`) is a single file that installs into Flatpak, sandboxed and
  updatable by installing a newer bundle, for people who prefer Flatpak to AppImage.
  `packaging/linux/flatpak-bundle.sh` repackages a `.tar.gz` with
  `packaging/linux/flatpak/com.localpaint.app.bundle.yml` on the freedesktop 26.08
  runtime. It does no Rust build and needs no network inside `flatpak-builder`, so the bundle
  holds the same binaries as the other formats. It takes the tarball as its input, so run
  `packaging/linux/package.sh --formats tar` first; install the bundle and run
  `localpaint-cli --version` inside the sandbox as a smoke test. The bundle names Flathub as its
  runtime repo, so users install
  it with:

  ```sh
  flatpak install --user localpaint-<v>-linux-x86_64.flatpak   # pulls org.freedesktop.Platform//26.08 from Flathub if missing
  flatpak run com.localpaint.app
  ```

  Sandbox permissions (justified in the manifest): Wayland with X11 fallback, IPC (X11
  shared memory), `dri` for the GPU, the CUPS socket, Avahi and `~/.cups` (read-only) for
  printing, and read/write access to Pictures and Documents. Every
  other file goes through the file-chooser and document portals. Network access enables update
  checks and makes the optional loopback `--control` server reachable from the host. Host fonts
  are read from `/run/host/fonts` and `/run/host/user-fonts`. Locally (on Linux, with `flatpak` and
  `flatpak-builder`): `packaging/linux/package.sh --formats tar && packaging/linux/flatpak-bundle.sh`.
- **Flathub**: `packaging/linux/flatpak/com.localpaint.app.yml` builds from source and
  is ready for a Flathub submission (it keeps the same runtime and `finish-args` as the bundle
  manifest). CI doesn't build it. A real build needs vendored crate
  sources (`cargo-sources.json` from `flatpak-cargo-generator.py`) and adds 20+ minutes per
  architecture. Build it by hand with the commands in the manifest's header.

All Linux binaries are built on Ubuntu 22.04 and need **glibc ≥ 2.35**: Ubuntu 22.04+,
Debian 12+, Fedora 36+, RHEL 10, openSUSE Tumbleweed. They link only glibc and libgcc_s. X11,
Wayland, xkbcommon, Vulkan and EGL are loaded at runtime from the system, which is also where
the GPU driver has to come from. That's why the .deb and .rpm declare them as dependencies
(the full list, and why each is there, is in `packaging/linux/nfpm.yaml`) and the AppImage
doesn't bundle them. The Flatpak gets them from the freedesktop runtime. Because the AppImage
and the tarball can't declare dependencies, `localpaint` checks for the libraries its session
(X11 or Wayland) needs before it opens a window (`apps/localpaint/src/linux_libs.rs`) and, if
one is missing, prints the package to install and exits with status 1 instead of crashing.
`LOCALPAINT_SKIP_LIB_CHECK=1` skips the check.

Locally (on Linux): install [nfpm](https://nfpm.goreleaser.com/install/), then
`packaging/linux/package.sh` (or `--formats "deb tar"`).

### FreeBSD

GitHub has no FreeBSD runners and this repository's workflow doesn't build FreeBSD, so
`packaging/freebsd/package.sh` runs on a FreeBSD 14.3 machine or VM with `bash`. The script builds both
binaries with `CARGO_BUILD_JOBS=4` (more runs a small VM out of memory) and writes
`localpaint-<v>-freebsd-x86_64.tar.gz`. FreeBSD's `uname -m` says `amd64`; the file name uses
`x86_64` like the other artifacts. The tarball is a `/usr/local`-style tree: `bin/localpaint`,
`bin/localpaint-cli`, and under `share/` the same desktop entry, MIME type, AppStream metainfo
and hicolor icons as Linux, plus the licences, `NOTICE` and `ATTRIBUTION.md` in
`share/doc/localpaint/`, and `localpaint-cli`'s man pages (`man/man1/`) and bash, zsh and fish
completions (`share/bash-completion`, `share/zsh/site-functions`, `share/fish`), written by the
`man-and-completions` example built with the binaries. Users install it with:

```sh
pkg install libxkbcommon wayland libX11 libXcursor libXrandr libXi libxcb mesa-libs vulkan-loader gtk3 fontconfig freetype2 alsa-lib
tar -xzf localpaint-<v>-freebsd-x86_64.tar.gz --strip-components 1 -C /usr/local
```

The script signs nothing and reads no secrets. The version, build date and commit come from the
environment (`LOCALPAINT_VERSION`, `LOCALPAINT_BUILD_SHA`, `LOCALPAINT_BUILD_DATE`, filled from
`git` when unset). The binaries aren't signed: FreeBSD has no code-signing scheme for loose
binaries.

Locally, on any OS, `packaging/freebsd/package.sh --dry-run` stages the tree from stub binaries
and lists the tarball, which checks the layout without a FreeBSD machine.

### Web

`packaging/web/package.sh` runs `trunk build --release` for the app (`apps/localpaint-web`, into
`dist/web`) and for the module that starts it (`apps/localpaint-web-boot`, into `dist/web-boot`).
It gzips the app's wasm, cuts it into parts of at most 20 MiB and zips the boot module's page with
the parts, sample `_headers` and `.htaccess` files and the hosting guide. The site only uses
relative URLs, so it works under any path and in an iframe. The wasm builds with the
size-optimized `wasm-release` Cargo profile (set in `apps/localpaint-web/index.html`), and the
script fails if any file in the site exceeds 24 MiB, below Cloudflare's 25 MiB per-file limit;
the app's wasm itself may be larger, since it ships as parts.
[`packaging/web/README.md`](../packaging/web/README.md) covers MIME types, compression,
caching, the iframe snippet and the `?webgl` / `?cpu` flags.

## Signing

This repository holds **no signing secrets** and its `release.yml` signs nothing: the artifacts it
publishes are unsigned builds from the scripts in `packaging/`. The packaging scripts still know how
to sign when the material is in the environment — `packaging/windows/sign.ps1` for `signtool`, and
`packaging/macos/package.sh` with a Developer ID — so a signed build is a local run with those
variables set, not a CI feature here.

| Variable | Used for |
|---|---|
| `WINDOWS_CERTIFICATE` | base64 `.pfx` code-signing certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | password for that `.pfx` |
| `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` | service principal for Azure Trusted Signing (an alternative to the `.pfx`) |
| `AZURE_SIGNING_ENDPOINT` | e.g. `https://eus.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE` | Trusted Signing account and certificate profile names |
| `APPLE_CERTIFICATE` | base64 `.p12` with a "Developer ID Application" identity |
| `APPLE_CERTIFICATE_PASSWORD` | password for that `.p12` |
| `KEYCHAIN_PASSWORD` | password for the temporary keychain (random if unset) |
| `APPLE_ID` | Apple ID used by `notarytool` |
| `APPLE_PASSWORD` | app-specific password for that Apple ID |
| `APPLE_TEAM_ID` | the Developer ID team |

`GITHUB_TOKEN` creates the release. Only the `publish` job gets `contents: write`.

On macOS, `packaging/macos/import-cert.sh` decodes the `.p12` into a temporary keychain
(`security create-keychain`, then `security import`, then `set-key-partition-list` so codesign
can use the key without a prompt). It exports the identity's SHA-1 as `MACOS_SIGN_IDENTITY`.
The keychain is deleted at the end.

## Icons

`assets/app-icon/localpaint.svg` is the canonical icon: the suite owner's drawing of a painter's
palette crossed by a brush, on an amber gradient tile, vectorised (MIT OR Apache-2.0, see
`LICENSE.txt` there; palette and geometry in `README.md`). `packaging/icons.sh` regenerates the
1024 px PNG, the `.icns`, the `.ico` (packed by
`cargo xtask ico`) and the hicolor PNGs from it. It needs `resvg`, plus `iconutil` on macOS.
The outputs are committed, so packaging never needs those tools.

## Checks

`.github/workflows/ci.yml` runs fmt, clippy, build and test on Ubuntu and Windows, a `makepkg
--printsrcinfo` parse of the Arch `PKGBUILD`, and the git-sources gate. The packaging scripts also
carry their own checks, which you run by hand on a change to `packaging/`: `packaging/windows/check-icons.ps1`,
`packaging/linux/apprun-test.sh`, `packaging/linux/apprun-cli-test.sh`,
`packaging/macos/verify.sh`, and a YAML check that the two Flatpak
manifests agree on the runtime and permissions.
