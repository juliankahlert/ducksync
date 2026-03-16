# ducksync
A duck DNS sync app with REST and DBus API 

## Installation

Pre-built binaries and packages are published to
[GitHub Releases](https://github.com/juliankahlert/ducksync/releases).

### Release assets

Each tagged release contains **7 assets**:

| Asset | Format | Arch |
|-------|--------|------|
| `ducksync_<VERSION>-1_amd64.deb` | Debian package | x86_64 |
| `ducksync_<VERSION>-1_arm64.deb` | Debian package | aarch64 |
| `ducksync-<VERSION>-1.x86_64.rpm` | RPM package | x86_64 |
| `ducksync-<VERSION>-1.aarch64.rpm` | RPM package | aarch64 |
| `ducksync-x86_64-unknown-linux-musl` | Static binary | x86_64 |
| `ducksync-aarch64-unknown-linux-musl` | Static binary | aarch64 |
| `SHA256SUMS` | Checksums | — |

> `SHA256SUMS` contains six entries — one for every asset except itself.

### Install via `.deb`

Replace `$TAG` with the release tag (e.g. `v0.1.0`) and `$VERSION` with the
version number (e.g. `0.1.0`).

**amd64:**

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/download/$TAG/ducksync_${VERSION}-1_amd64.deb" -o ducksync.deb
sudo dpkg -i ducksync.deb
```

**arm64:**

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/download/$TAG/ducksync_${VERSION}-1_arm64.deb" -o ducksync.deb
sudo dpkg -i ducksync.deb
```

### Install via `.rpm`

**x86_64:**

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/download/$TAG/ducksync-${VERSION}-1.x86_64.rpm" -o ducksync.rpm
sudo rpm --install ducksync.rpm
```

**aarch64:**

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/download/$TAG/ducksync-${VERSION}-1.aarch64.rpm" -o ducksync.rpm
sudo rpm --install ducksync.rpm
```

### Raw binary

Download the latest static musl binary directly:

**x86_64:**

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/latest/download/ducksync-x86_64-unknown-linux-musl" -o ducksync
chmod +x ducksync
```

**aarch64:**

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/latest/download/ducksync-aarch64-unknown-linux-musl" -o ducksync
chmod +x ducksync
```

### Verify checksums

```sh
curl -fsSL "https://github.com/juliankahlert/ducksync/releases/download/$TAG/SHA256SUMS" -o SHA256SUMS
sha256sum -c SHA256SUMS
```

This verifies the six release assets listed in the checksum file.
