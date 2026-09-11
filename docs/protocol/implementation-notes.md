# Implementation Notes

Notes for extending `qcyx_core`, plus a running list of confirmed vs. open items.

---

## Required Libraries

| Purpose | Crate |
|---|---|
| BLE central/GATT | `btleplug` `0.13` |
| UUID handling | `uuid` `1.26` |
| Async runtime | `tokio` (full) |
| Notification stream combinators | `futures` `0.3` |

`btleplug` abstracts CoreBluetooth (macOS), BlueZ (Linux), and WinRT (Windows). WinRT needs the most defensive handling — see [error-recovery.md](./error-recovery.md).

---

## Reverse-Engineering Method

Strictly black-box: BLE traffic observation only, never disassembling QCY software. In order of trust:

1. **Direct BLE capture** — ground truth for every confirmed value.
2. **Third-party repos** (`hui1601/Quicky`, `HttpKiwi/OpenQCY`, `pedro-labsabs/521C`, `GiorgioRafael/OpenQCY-Desktop`) — orientation only. These introduced real bugs when trusted directly (wrong opcode `0x0C` for ANC, incorrect scene tables). Nothing from them ships without independent confirmation.

Workflow for new protocol surface (new opcode, scene, event type): capture first, third-party repos for orientation only, then a regression test pinned to the exact captured bytes (e.g. `command::tests::anc_scene_normal_matches_capture`).

---

## Portability Checklist (New QCY Model)

Re-verify per model rather than assuming carryover from the HT08:

- [ ] Manufacturer Company ID (`0x521C` on HT08)
- [ ] GATT service UUID (`0xa001` on HT08)
- [ ] Command/notify characteristic UUIDs (`0x1001`/`0x1002` on HT08)
- [ ] Whether a second service reuses the same characteristic numbers (confirmed on HT08 as `0x7033`)
- [ ] Whether `0x17` or legacy `0x0C` is the live ANC opcode
- [ ] ANC scene parameter triplets
- [ ] Whether `ANC_RESULT` (`0x28`) is sent at all, or only an echo

---

## Windows-Specific Defensive Patterns

Full detail in [error-recovery.md](./error-recovery.md).

| Pattern | Why |
|---|---|
| Connect + liveness retry | WinRT can report `connect()` `Ok` then silently drop the link |
| Service discovery retry with backoff | WinRT's GATT cache can race `discover_services()` |
| Longer ANC confirmation timeout (10s) | Windows BLE notification latency/jitter is worse; up to ~6.5s observed for `Adaptive` |
| Filter characteristic lookups by `service_uuid`, not just `uuid` | Characteristic numbers are reused across services |
| `unsubscribe()` before every `subscribe()` | Clears stale CCCD state |
| Subscribe to every notify characteristic, not just decoded ones | Matches the connection pattern firmware expects |

---

## BlueZ `mtu()` Panic

`btleplug` 0.13's BlueZ backend unwraps the characteristic MTU inside `Peripheral::mtu()`, and older bluetoothd releases don't expose it — the call panics there, which aborts release builds (`panic = "abort"`). `DeviceHandle::write_type_for` only calls `mtu()` for payloads larger than the spec-minimum ATT payload (20 bytes), so every command except the EQ table and a long rename avoids it. The remaining exposure needs an upstream fix.

---

## Roadmap / Open Questions

- **Opcode `0x3B` (Music Info)** — referenced in third-party repos, not captured.
- **`color_index` field** — referenced in advertisement-parsing code, not confirmed.
- **`Event::Alarms` / `Event::AncWear` decoders** — planned, pending capture.
- **`CMDID_MOREDEVICE` (`0x24`, dual-device connection)** — writes `24 01 01` / `24 01 02` observed outside a documented action sequence. Unverified lead, not a confirmed feature.
- **CLI/GUI connection duplication on Windows** — the CLI opens its own BLE connection independent of any existing OS audio connection, which can make the device appear as two separate profiles in the OS device list. Understood as a platform constraint, not fixable purely in this crate.

**Confirmed/implemented:** ANC (full submenu), battery, firmware version, channel balance, EQ presets, and the three actions in [device-actions.md](./device-actions.md). See [packet-reference.md](./packet-reference.md) for the full opcode table.

---

## Feature Coverage Map

### Battery status
✅ Fully covered — see [battery.md](./battery.md). Case doesn't report a live level on this model.

### Sound controls

- **EQ presets** (7 built-in): ✅ Implemented — see [eq.md](./eq.md). Custom per-band editing: ❌ not implemented (142-byte table structure unconfirmed).
- **Noise Cancelling** (Adaptive, Indoor/Daily Commute/Noisy ×3 levels, Wind Resistance): ✅ Fully implemented.
- **Transparency**: ✅ Both sub-states (vocal enhancement, ambient sound) implemented.
- **Normal**: ✅ Confirmed.
- **Channel balance**: ✅ Implemented — see [balance.md](./balance.md).

### Device settings

| Feature | Status | Notes |
|---|---|---|
| Click actions (tap gestures) | ❌ Not implemented | Opcode `0x2B` referenced in third-party repos; exact write framing not captured |
| Notification volume | ⚠️ Partial | `0x1D` captured (`1d 02 08 0f`), meaning unconfirmed |
| Scheduled shutdown | ✅ Format known | `0x14`, 16-bit minutes; cross-checked disabled state (`14 04 ff ff 00 00`); not wired in |
| Wear detection | ✅ Format + value confirmed | `0x2C`, e.g. `2c 04 00 01 01 01`; good next candidate |
| Disconnect / power off | ❔ Unclear opcode | `0x1F` captured (`1f 02 05 00`), meaning unconfirmed |
| Game mode | ❌ Not implemented | Opcode `0x4A` referenced in third-party repos, nothing captured |
| Sleep mode | ⚠️ Opcode captured | `0x10` (`10 01 00`), likely boolean, unconfirmed |
| LDAC switch | ⚠️ Opcode captured | `0x23` (`23 01 00`), meaning unconfirmed |
| Dual device connection | ⚠️ Opcode captured | `0x24`, two values seen outside documented sequence |
| Firmware update | ❌ Out of scope | Likely a full OTA transfer flow |
| Find my earbuds | ❌ Not implemented | No confirmed opcode |
| Device name | ✅ Read implemented | Advertised name: no protocol work needed. Pairing name (`0x18`) read confirmed but not wired in. Write (rename) implemented — see [device-actions.md](./device-actions.md) |
| Reset to default | ✅ Implemented | See [device-actions.md](./device-actions.md) |
| Factory reset | ✅ Implemented | See [device-actions.md](./device-actions.md) |
| Device info (Product ID, model, brand, MAC, firmware version) | ⚠️ Partial | Firmware version implemented ([version.md](./version.md)); MAC available via `btleplug` but not surfaced; Product ID/model/brand not located — may require inspecting raw advertisement bytes rather than ATT-layer captures |
