# Device Settings & Parameter Query

Six settings-screen opcodes, plus the `0xFE` query mechanism the official app
uses to read any of them (and the earlier-documented `0x18`) on screen load.
All confirmed by timestamped user actions against a live capture.

---

## Parameter Query (`0xFE`)

Not part of [`read_state_sync`](./anc.md#reading-the-current-scene-at-connect-time)'s
single-blob read — every opcode below is fetched with its own request/reply
round trip when its settings screen opens.

```
Write:  ff 03 fe 01 <target_opcode>
Read:   <target_opcode> <param_len> <payload...>   (on notify)
```

| Item | Value |
|---|---|
| Opcode | `0xFE` |
| Params | 1B: the opcode to query |

`crate::query::request` builds this; `DeviceHandle::query_param` sends it and
waits on notify for a block carrying the same opcode back, returning its raw
parameter bytes.

---

## Notification Volume (`0x1D`)

```
Write:  ff 04 1d 02 <level> 00
Read:   1d 02 <level> 0f
```

| Item | Value |
|---|---|
| Opcode | `0x1D` |
| Params | 2B: `level`, then `0x00` on write / `0x0F` on the notify reply |
| Levels | `0x04` low, `0x06` medium, `0x08` high, `0x0A` max |

`crate::notification_volume`. Levels confirmed by cycling through all four
in the app.

---

## Scheduled Power-Off (`0x14`)

An idle-independent countdown, armed regardless of playback or connection
state. Distinct from [Disconnect Power-Off](#disconnect-power-off-0x1f)
below.

```
Write:  ff 06 14 04 <lo> <hi> 00 00
Read:   14 04 <lo> <hi> <lo> <hi>          (echoed twice when armed)
        14 04 ff ff 00 00                  (disabled)
```

| Item | Value |
|---|---|
| Opcode | `0x14` |
| Params | 4B: little-endian `u16` minutes, then two `0x00` bytes |
| Disabled | `0xFFFF` |
| App presets | 15 / 30 / 60 / 90 minutes, plus a free-form custom field |

`crate::scheduled_power_off`. Confirmed with all four presets and a custom
value of 183 minutes (`0xB7 0x00`).

---

## Disconnect Power-Off (`0x1F`)

Starts only once the Bluetooth connection is lost; powers off after it
elapses unless reconnected first.

```
Write:  ff 04 1f 02 <lo> <hi>
Read:   1f 02 <lo> <hi>
```

| Item | Value |
|---|---|
| Opcode | `0x1F` |
| Params | 2B: little-endian `u16` minutes — no echo pair, unlike `0x14` |
| Never | `0xFFFF` |
| App presets | 5 (default) / 10 / 30 / 60 minutes, or never |

`crate::disconnect_power_off`. Confirmed with all four presets and the
"never" state; `0x05 0x00` (5 minutes) is the device's connect-time default.

---

## Wear Detection (`0x2C`)

In-ear detection, plus a second toggle the app labels "ANC" that reappears
once wear detection is on: re-applies the last ANC scene when the earbuds
detect they've been put back in.

```
Write:  ff 05 2c 03 <wear> 01 <anc>
Read:   2c 04 <wear> 01 <anc> <status>
```

| Item | Value |
|---|---|
| Opcode | `0x2C` |
| Params (write) | 3B: `wear` (`0x00`/`0x01`), `0x01` reserved, `anc` (`0x00`/`0x01`) |
| Params (read) | 4B: same three, plus a trailing status byte |

`crate::wear_detection`. Confirmed by toggling the main switch and the ANC
sub-toggle independently — each flag moves on its own write, the other byte
holds. Connect-time default observed: wear detection off, ANC sub-toggle
already on (`00 01 01 00`).

---

## Game Mode (`0x09`)

```
Write:  ff 03 09 01 <state>
Read:   09 01 <state>
```

| Item | Value |
|---|---|
| Opcode | `0x09` |
| Params | 1B: `0x01` on, `0x02` off |

`crate::game_mode`. Echoed on notify immediately after each write. Device
default is off (`0x02`).

---

## Sleep Mode (`0x10`)

```
Write:  ff 03 10 01 <state>
```

| Item | Value |
|---|---|
| Opcode | `0x10` |
| Params | 1B: `0x01` on, `0x02` off |

`crate::sleep_mode`. Same encoding as Game Mode, on a different opcode. No
notify echo was observed for either write in the capture this was confirmed
against — `DeviceHandle::set_sleep_mode` doesn't wait for one.

---

## Notes

None of these six writes were observed to trigger the two-stage
echo-then-async-result flow that [ANC](./anc.md#confirmation-flow) has.
Where an echo was observed at all (`0x14`, `0x1F`, `0x09`), it's a single
notification with the same shape as the write; `0x10` produced none in this
capture. All six follow Balance's fire-and-forget pattern rather than ANC's.
