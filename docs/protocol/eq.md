# Equalizer

All 7 named presets confirmed.

---

## Opcode

| Opcode | Name | Direction | Status |
|---|---|---|---|
| `0x22` | `MULTIEQ2` | Write; also readable via state-sync blob | ✅ Confirmed |

Same command characteristic (`0x1001`) and `0xFF` framing as ANC and balance.

---

## Wire Format

```
ff <body_len> 22 <param_len> <preset_id> <142-byte table>
```

`param_len` is always `0x8f` (143 = 1 ID byte + 142 table bytes) — there's no "ID only" shortcut, selecting a preset means sending its entire table.

### Preset IDs

| Preset | ID |
|---|---|
| Spatial Audio | `0x0a` |
| Default | `0x01` |
| Popular | `0x02` |
| Bass Boost | `0x03` |
| Rock | `0x04` |
| Soft | `0x05` |
| Classic | `0x06` |
| Custom (per-band editing) | `0x80` |

Spatial Audio and Default share a byte-for-byte identical table — Spatial Audio likely layers a separate DSP effect rather than changing the EQ curve.

---

## The 142-Byte Table

Not understood at the byte level, and not safe to hand-edit. A single per-band edit changes dozens of bytes at once, not a handful near one offset — consistent with packed filter coefficients (frequency/Q/gain or biquad sets) recomputed across the whole table, not a simple per-band gain array.

- Resetting inside custom mode snaps the table back to the previously-active preset's bytes, but keeps the ID tag `0x80`.
- **No confirmation** — unlike ANC, no echo or result notification follows a `0x22` write (fire-and-forget, same as [balance](./balance.md)).

`qcyx_core::eq::EqPreset` only covers the 7 built-in presets, replaying their captured bytes exactly. Custom per-band editing is not implemented — reconstructing the coefficient encoding would need much more isolated, targeted data.

---

## Implementation

`EqPreset` is a Rust enum, one variant per preset, each carrying its ID and full table as `'static` constants. `qcyx_core::eq::set_preset` concatenates ID + table and writes it. `DeviceHandle::set_eq_preset` sends and returns immediately (no confirmation to await).
