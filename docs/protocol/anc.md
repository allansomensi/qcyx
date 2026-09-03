# ANC (Noise Control)

The most thoroughly confirmed control surface: every top-level mode, every Transparency sub-state, and the full Noise Cancelling submenu.

---

## Opcodes

| Opcode | Name | Direction | Status |
|---|---|---|---|
| `0x17` | `ANC_SETTING` | Write + echoed on notify | ✅ Confirmed |
| `0x28` | `ANC_RESULT` | Notify only | ✅ Confirmed |
| `0x0C` | `NoiseCancelMode` (legacy) | Write | ⚠️ Ignored by HT08 firmware — no echo, no error |

Several third-party repos document `0x0C` as the ANC command; on the HT08 it's silently ignored. `0x17` is the correct primary path. `0x0C` is kept only as a documented legacy fallback for other models — not assumed correct without its own confirmation.

---

## ANC Scene Table

Each scene is a 3-byte triplet `(mode, sub_scene, noise_value)` written on opcode `0x17`.

| Scene | `mode` | `sub_scene` | `noise_value` |
|---|---|---|---|
| Normal | `0x02` | `0x00` | `0x00` |
| Transparency, vocal enhancement | `0x03` | `0x02` | `0x00` |
| Transparency, ambient sound (level `L`) | `0x03` | `0x01` | `L` (raw byte, range unconfirmed) |
| Noise Cancelling → Adaptive | `0x01` | `0x05` | `0x00` |
| Noise Cancelling → Indoor, level 1/2/3 | `0x01` | `0x01` | `0x00`/`0x01`/`0x02` |
| Noise Cancelling → Daily Commute, level 1/2/3 | `0x01` | `0x02` | `0x00`/`0x01`/`0x02` |
| Noise Cancelling → Noisy, level 1/2/3 | `0x01` | `0x03` | `0x00`/`0x01`/`0x02` |
| Noise Cancelling → Wind Resistance | `0x01` | `0x04` | `0x00` |

`mode = 0x02` is **Normal**, not "Noise Cancelling" — despite the byte pattern suggesting otherwise. The real Noise Cancelling mode is `mode = 0x01`.

"Off" and "Outdoor" ANC scenes are not yet confirmed and remain unimplemented.

### Transparency's Vocal Enhancement Toggle

- **Enabled** (default): `sub_scene = 0x02`, `noise_value = 0x00`.
- **Disabled**: `sub_scene = 0x01`, `noise_value` becomes a direct ambient-sound intensity level. Only `0x01`, `0x03`, `0x06` have been observed, confirming the mechanism but not the slider's true min/max — `TransparencyMode::AmbientSound` takes an unclamped `u8`.

### Noise Cancelling Submenu

Five sub-modes under `mode = 0x01`, keyed by `sub_scene`. Indoor, Daily Commute, and Noisy additionally take a 1–3 intensity level, zero-indexed in `noise_value` (level 1 → `0x00`, level 2 → `0x01`, level 3 → `0x02`). Adaptive and Wind Resistance have no level control (`noise_value` fixed at `0x00`).

**Adaptive is notably slower to confirm** — `ANC_RESULT` can take up to ~6.5s, vs. ~2–3s for other scenes (`ANC_CONFIRM_TIMEOUT` budgets for this).

---

## Confirmation Flow

A single ANC write produces two separate, asynchronous notifications — not a single request/response:

```
host                                   device
  │──── write ANC_SETTING (0x17) ───────▶│
  │◀─── notify: echo of 0x17 ────────────│   (near-immediate)
  │        ... 2–3s later (up to ~6.5s    │
  │            for Adaptive) ...          │
  │◀─── notify: ANC_RESULT (0x28) ───────│   (async confirmation)
```

1. **Echo** — the device notifies back the same `0x17` opcode and params, confirming the write was received.
2. **`ANC_RESULT` (`0x28`)** — a later one-byte notification: `0x01` = applied by firmware. No failure byte value has been observed; anything other than `0x01` is treated as "not confirmed."

### Why Two Signals Matter

This crate distinguishes four outcomes (`AncConfirmation` in `qcyx_core::client`):

| Outcome | Meaning |
|---|---|
| `Applied` | `ANC_RESULT` arrived with `0x01` |
| `Rejected` | `ANC_RESULT` arrived with a non-`0x01` value |
| `EchoedOnly` | Echo arrived, but no `ANC_RESULT` before timeout — link is likely fine, confirmation was slow/dropped/unsent |
| `NoResponse` | Nothing arrived at all — likely a notify-subscription failure or (Windows) missing BLE bonding. See [error-recovery.md](./error-recovery.md) |

---

## Response Payload Layout

**Echo (`0x17` on notify)** — same 3-byte layout as the write: `mode (1B) | sub_scene (1B) | noise_value (1B)`.

**`ANC_RESULT` (`0x28` on notify)** — `applied (1B)`, `0x01` = success.

Both decode via `qcyx_core::response::dispatch` into `Event::AncSetting` / `Event::AncResult`; unrecognized opcodes fall back to `Event::Raw` rather than being dropped.

---

## Reading the Current Scene at Connect Time

A plain GATT Read Request (ATT `0x0A`) against the notify characteristic's own value, done once after subscribing. The response is one `0xFF`-framed packet holding several back-to-back command blocks (the device's full current settings), parsed the same way as any multi-command packet:

```
ff 30 2c 04 00 01 01 01 08 03 24 24 0f 09 01 02
   17 03 03 02 00 10 01 00 16 01 32 04 01 02 23
   01 00 24 01 00 1f 02 05 00 1d 02 08 0f 19 04
   57 51 30 30
```

The `17 03 03 02 00` block is `ANC_SETTING`, params `03 02 00` — round-trips via `AncScene::from_triplet` to `Transparency(VocalEnhancement)`. `DeviceHandle::read_state_sync` performs this read and extracts both the ANC block and the balance block (see [balance.md](./balance.md)); the rest of the blob (button config, tone volume, standby timer, pairing name, ...) isn't decoded yet.

`crate::session::ensure_connected` performs this read once, at session start, to seed `AppState`'s active scene. The CLI's one-shot `client::connect()` skips it — nothing in the CLI UI needs it, and it would add latency to every invocation.

A response whose `17` block doesn't match a known `AncScene` triplet resolves to `None` rather than a guess.
