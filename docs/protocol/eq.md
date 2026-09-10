# Equalizer

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

### Built-in presets

Packed filter coefficients (frequency/Q/gain or biquad sets), not a simple per-band gain array — not safe to hand-edit. `qcyx_core::eq::EqPreset` covers the 7 built-in presets as fixed `'static` tables.

- Resetting inside custom mode snaps the table back to the previously-active preset's bytes, but keeps the ID tag `0x80`.
- **No confirmation** — unlike ANC, no echo or result notification follows a `0x22` write (fire-and-forget, same as [balance](./balance.md)).

### Custom (per-band) curve

The moment any band is edited, the table switches to a distinct, simpler per-band record format — not an extension of the preset-coefficient encoding above.

10 fixed-size records (7 bytes each, 70 bytes total), one per band, in ascending frequency order:

```
[0x00, marker, freq_lo, freq_hi, gain_lo, gain_hi, 0x64]
```

| Field | Size | Notes |
|---|---|---|
| `0x00` | 1 | constant |
| `marker` | 1 | `0x00` for band 0 (31 Hz, low-shelf); `0xFF` for every other band (peaking/high-shelf). Constant per band, independent of gain. |
| `freq_lo/freq_hi` | 2 | center frequency in Hz, little-endian `u16` |
| `gain_lo/gain_hi` | 2 | `gain_db * 100`, little-endian **signed** `i16` |
| `0x64` | 1 | constant (100), independent of gain |

Band order / center frequencies: `31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000` Hz.

Gain range: `-8..=8` dB (`qcyx_core::eq::CUSTOM_GAIN_MIN_DB`/`MAX_DB`).

Bytes past the 10th record (offset 71..142 of the table) are left zeroed by `qcyx_core::eq::set_custom`. The device does not require anything specific there.

---

## Implementation

`EqPreset` is a Rust enum, one variant per built-in preset, each carrying its ID and full table as `'static` constants. `qcyx_core::eq::set_preset` concatenates ID + table and writes it. `qcyx_core::eq::set_custom` builds the 10-record custom table from a `[i16; 10]` of per-band gains. Both go through `DeviceHandle::set_eq_preset` / `set_eq_custom`, which send and return immediately — no confirmation to await.

### Write type

This command (`opcode` + `param_len` + ID + table, up to 147 bytes total) must be sent as `WriteWithoutResponse`, after the ATT MTU has been negotiated above its default (23 bytes) — the characteristic does not reliably accept `WriteRequest` for a payload this size. `DeviceHandle::write_type_for` checks the peripheral's actual negotiated MTU (`Peripheral::mtu()`) rather than the unnegotiated default floor before picking the write type.
