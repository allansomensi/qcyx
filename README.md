<div align="center">
  <img width="150"  alt="QCYx Logo" src="https://github.com/user-attachments/assets/780bacd6-d685-4fba-9e43-af941aa66d7b" />

  # QCYx

  <p><em>Unofficial cross-platform desktop companion app for QCY earbuds.</em></p>

  [![Built with Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)](https://www.rust-lang.org/)
  [![Version](https://img.shields.io/github/v/release/allansomensi/qcyx?color=blue&label=version)](https://github.com/allansomensi/qcyx/releases)
  [![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

  <br/>
</div>

**QCYx** is an unofficial, **cross-platform companion tool** for configuring QCY wireless earbuds. Available as both a **command-line interface (CLI)** and a **graphical interface (GUI)**, it allows you to manage your device settings **entirely independently from any mobile app**.

---

## 🎯 Motivation

Configuration of QCY earbuds is officially restricted to the proprietary mobile application. Desktop operating systems currently lack a native client, requiring users to perform a Bluetooth handoff to a mobile device simply to modify device parameters.

**QCYx** addresses this limitation by implementing the QCY BLE GATT protocol for desktop environments. It provides a direct, offline interface to read and write hardware configurations directly from a PC, bypassing the need for the official mobile app.

---

## ✨ Features

> ⚠️ QCYx is in early development. Features marked 🚧 are planned but not yet implemented.

- ✅ **Active Noise Cancellation (ANC):** Full control over Normal, Transparency, and all specific ANC modes.
- ✅ **Battery Monitoring:** Read current battery status.
- ✅ **Audio Controls:** Set equalizer presets and adjust channel balance.
- ✅ **Notification Volume:** Set the earbuds' notification volume.
- ✅ **Power-Off Timers:** Scheduled power-off and power-off after disconnect.
- ✅ **Wear Detection:** Toggle in-ear detection.
- ✅ **Game Mode:** Toggle low-latency audio for gaming.
- ✅ **Sleep Mode:** Toggle sleep mode.
- ✅ **Click Actions:** Assign single/double/triple tap actions, per earbud.
- ✅ **LDAC:** High-resolution codec toggle.
- ✅ **Dual Device Connection:** Connect to two devices at once.
- ✅ **Device Management:** Read firmware version, rename the device, reset to default, and factory reset.
- ✅ **Multi-language Support:** UI available in English and Portuguese (pt-BR).
- 🚧 **Custom EQ:** Per-band editing.
- 🚧 **Find my earbuds:** Trigger a high-frequency sound to locate misplaced earbuds.
- 🚧 **Firmware update:** Flash official firmware updates over the air (OTA).

---

## ✅ Supported Devices

| Device | Status |
|---|---|
| MeloBuds Pro | ✅ Validated |
| MeloBuds N70 | 🚧 Planned |
| MeloBuds N60 | 🚧 Planned |
| MeloBuds N50 | 🚧 Planned |
| MeloBuds A30 | 🚧 Planned |

---

## 📥 Installation

You can find pre-built binaries, installers (`.msi`), and automated installation scripts (`.ps1`, `.sh`) on the [Releases page](https://github.com/allansomensi/qcyx/releases).

> **Windows Users:** If you encounter a "SmartScreen" warning when running the MSI, click **"More info"** → **"Run anyway"**. This is normal for unsigned open-source tools.

<details>
<summary><strong>Or install from source (Cargo)</strong></summary>

**Prerequisites:** Rust toolchain (`rustup`), `libusb` 1.0+.

```bash
git clone https://github.com/allansomensi/qcyx
cd qcyx
cargo build --release
```

The binary will be at `target/release/qcyx`.
</details>

---

## 💻 Usage

### CLI

The CLI connects, sends one command, and disconnects — a fresh session per invocation.

```bash
# Set ANC to normal (no ANC/transparency processing)
qcyx-cli anc normal

# Transparency: vocal enhancement or an ambient level
qcyx-cli anc transparency
qcyx-cli anc transparency --level 5

# The Noise Cancelling submenu
qcyx-cli anc noise-cancelling adaptive
qcyx-cli anc noise-cancelling wind
qcyx-cli anc noise-cancelling indoor 3
qcyx-cli anc noise-cancelling daily-commute 2
qcyx-cli anc noise-cancelling noisy 1

# Read battery status
qcyx-cli battery

# Read device name and firmware version
qcyx-cli version

# Set channel balance (0 = full left, 100 = full right, 50 = centered)
qcyx-cli balance 50

# Equalizer presets
qcyx-cli eq spatial
qcyx-cli eq default
qcyx-cli eq popular
qcyx-cli eq bass-boost
qcyx-cli eq rock
qcyx-cli eq soft
qcyx-cli eq classic

# Notification volume
qcyx-cli notification-volume low
qcyx-cli notification-volume medium
qcyx-cli notification-volume high
qcyx-cli notification-volume max

# Scheduled power-off timer (minutes, or "off" to disable)
qcyx-cli scheduled-power-off 30
qcyx-cli scheduled-power-off 183
qcyx-cli scheduled-power-off off

# Power-off-after-disconnect timer (minutes, or "never")
qcyx-cli disconnect-power-off 10
qcyx-cli disconnect-power-off never

# In-ear wear detection and its ANC-on-wear sub-toggle
qcyx-cli wear-detection --enabled
qcyx-cli wear-detection --enabled --anc-on-wear=false
qcyx-cli wear-detection

# Game mode
qcyx-cli game-mode on
qcyx-cli game-mode off

# Sleep mode
qcyx-cli sleep-mode on
qcyx-cli sleep-mode off

# LDAC codec
qcyx-cli ldac on
qcyx-cli ldac off

# Dual-device (multipoint) connection
qcyx-cli multipoint on
qcyx-cli multipoint off

# Touch actions — assign what a tap gesture does, per earbud/click-count
qcyx-cli touch-action left-single none
qcyx-cli touch-action left-single play-pause
qcyx-cli touch-action left-double previous
qcyx-cli touch-action right-double next
qcyx-cli touch-action left-triple voice-assistant
qcyx-cli touch-action right-triple anc
qcyx-cli touch-action right-single volume-up
qcyx-cli touch-action right-single volume-down
qcyx-cli touch-action left-single game-mode

# Reset settings to default
qcyx-cli reset-default

# Factory reset (asks for confirmation; skip with --yes)
qcyx-cli factory-reset

# Rename the device
qcyx-cli rename "My Earbuds"
```

### GUI

The GUI connects once on startup and stays connected for as long as it's running.

```bash
qcyx
```

---

## 🔬 How It Works

QCYx communicates with QCY earbuds over **Bluetooth Low Energy (BLE) GATT**, using the same proprietary `0xFF`-framed binary protocol as the official app.

The protocol was reverse-engineered **black-box** — primarily by capturing and analyzing BLE traffic (Wireshark) between a Melobuds Pro and the official QCY Android app, cross-referenced against findings from third-party open-source community repositories. No QCY source code, firmware, or proprietary SDKs were accessed or used at any point. The reverse engineering was done **strictly for the purpose of interoperability** with platforms not officially supported by QCY.

The full protocol is documented in [`docs/protocol/`](./docs/protocol/README.md).

---

## 🤝 Contributing

Contributions are very welcome, especially:

- Protocol research for additional QCY devices or undocumented BLE operations.
- Testing on different hardware and operating systems.
- GUI/CLI development.
- Documentation improvements and i18n translations.

Please open an issue before starting work on a large feature so we can coordinate.

---

## ⚖️ Legal

**QCYx is an unofficial, community-driven, open-source project. It is not affiliated with, endorsed by, sponsored by, or associated with QCY or Dongguan Hele Electronics Co., Ltd. in any way.**

"QCY" and its related product names are trademarks of Dongguan Hele Electronics Co., Ltd. All trademarks are the property of their respective owners and are used here strictly for descriptive and nominative purposes.

The reverse engineering performed in this project was conducted entirely black-box — by observing Bluetooth Low Energy (BLE) communication between the earbuds and the official mobile app — and is intended solely to enable interoperability on platforms not officially supported. No proprietary code, firmware, or trade secrets were accessed or incorporated.

QCYx is released under the [MIT License](./LICENSE).
