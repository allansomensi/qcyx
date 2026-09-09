# Packet Reference

Master table of all known opcodes, in the `0xFF`-framed format ([frame-format.md](./frame-format.md)).

---

## Confirmed Opcodes

| Opcode | Name | Direction | Params | Status | Doc |
|---|---|---|---|---|---|
| `0x01` | `RESET_DEFAULT` | Write | None | ⚠️ Framing confirmed, not wire-verified | [device-actions.md](./device-actions.md) |
| `0x03` | `FACTORY_RESET` | Write | None | ⚠️ Framing confirmed, not wire-verified | [device-actions.md](./device-actions.md) |
| `0x09` | Game mode | Write + echoed on notify | 1B: `0x01` on / `0x02` off | ✅ Confirmed | [device-settings.md](./device-settings.md#game-mode-0x09) |
| `0x10` | Sleep mode | Write | 1B: `0x01` on / `0x02` off | ✅ Confirmed | [device-settings.md](./device-settings.md#sleep-mode-0x10) |
| `0x14` | Scheduled power-off | Write; read via `0xFE` request | 4B: `u16` LE minutes + 2 zero bytes; `0xFFFF` = disabled | ✅ Confirmed | [device-settings.md](./device-settings.md#scheduled-power-off-0x14) |
| `0x16` | `BALANCE` | Write; read via state-sync blob | 1B: value (`0..=100`, `50` = centered) | ✅ Confirmed | [balance.md](./balance.md) |
| `0x17` | `ANC_SETTING` | Write + echoed on notify | 3B: `mode`, `sub_scene`, `noise_value` | ✅ Confirmed | [anc.md](./anc.md) |
| `0x18` | `PAIRNAME` | Write (rename); read via `0xFE` request | Length-prefixed UTF-8 name | ✅ Read confirmed; write framing only | [device-actions.md](./device-actions.md), [version.md](./version.md) |
| `0x1D` | Notification volume | Write; read via `0xFE` request | 2B: level + status byte | ✅ Confirmed | [device-settings.md](./device-settings.md#notification-volume-0x1d) |
| `0x1F` | Disconnect power-off | Write; read via `0xFE` request | 2B: `u16` LE minutes; `0xFFFF` = never | ✅ Confirmed | [device-settings.md](./device-settings.md#disconnect-power-off-0x1f) |
| `0x22` | `MULTIEQ2` | Write; read via state-sync blob | 1B preset ID + 142B table | ✅ Confirmed, all 7 presets | [eq.md](./eq.md) |
| `0x28` | `ANC_RESULT` | Notify only | 1B: `applied` (`0x01` = success) | ✅ Confirmed | [anc.md](./anc.md) |
| `0x2C` | Wear detection | Write; read via `0xFE` request | 3B write / 4B read: `wear`, reserved, `anc_on_wear`(, status) | ✅ Confirmed | [device-settings.md](./device-settings.md#wear-detection-0x2c) |
| `0xFE` | Parameter query (GET) | Write | 1B: target opcode | ✅ Confirmed | [device-settings.md](./device-settings.md#parameter-query-0xfe) |

## Legacy / Superseded

| Opcode | Name | Status | Notes |
|---|---|---|---|
| `0x0C` | `NoiseCancelMode` | ⚠️ Ignored by HT08 firmware | Kept as documented fallback for other models — see [anc.md](./anc.md#opcodes) |

## Known-But-Unimplemented

Referenced in third-party repos, not yet confirmed against real traffic:

| Opcode | Name | Notes |
|---|---|---|
| `0x3B` | Music Info | Not captured or implemented |

Also unconfirmed: a `color_index` field on BLE advertisement payloads, and `Event::Alarms` / `Event::AncWear` decoders in `response.rs`.

---

## Non-Opcode Characteristics

Plain GATT characteristics, read directly — not part of the `0xFF`-framed protocol.

| UUID | Name | Status | Notes |
|---|---|---|---|
| `00000008` | Battery | ✅ Confirmed | 3 bytes `[left, right, case]`, bit 7 = charging. `case` reads 0% on HT08 — see [battery.md](./battery.md) |
| `00000007` | Version | ⚠️ Layout unverified | 6 bytes: left + right `major.minor.patch` |
| `0000000B` | EQ | ❌ Not implemented | Payload format unconfirmed |
| `0000000D` | Button config | ❌ Not implemented | Payload format unconfirmed |

Device *name* isn't in this table — no GATT characteristic needed, see [version.md](./version.md#device-name-not-wire-protocol).

## ANC Scene Parameter Reference

Full detail in [anc.md](./anc.md#anc-scene-table).

| Scene | `mode` | `sub_scene` | `noise_value` |
|---|---|---|---|
| Normal | `0x02` | `0x00` | `0x00` |
| Transparency, vocal enhancement | `0x03` | `0x02` | `0x00` |
| Transparency, ambient sound | `0x03` | `0x01` | raw level (range unconfirmed) |
| Noise Cancelling → Adaptive | `0x01` | `0x05` | `0x00` |
| Noise Cancelling → Indoor | `0x01` | `0x01` | `0x00`/`0x01`/`0x02` |
| Noise Cancelling → Daily Commute | `0x01` | `0x02` | `0x00`/`0x01`/`0x02` |
| Noise Cancelling → Noisy | `0x01` | `0x03` | `0x00`/`0x01`/`0x02` |
| Noise Cancelling → Wind Resistance | `0x01` | `0x04` | `0x00` |
