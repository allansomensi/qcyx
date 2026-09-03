# Battery

Not part of the `0xFF`-framed command protocol — a plain GATT characteristic.

---

## Characteristic

| Item | Value |
|---|---|
| UUID | `00000008-0000-1000-8000-00805f9b34fb` |
| Properties | `READ`, `NOTIFY` |
| Service | `0000a001-...` |

Read directly via ATT Read Request (`0x0A`) — no opcode, no `Command` framing. `NOTIFY`-capable, so it's picked up by the "subscribe to every notify characteristic" sweep (see [transport.md](./transport.md)); this crate currently only reads on demand rather than acting on unsolicited notifications.

---

## Payload Layout

3 raw bytes, always `[left, right, case]`. A payload shorter than 3 bytes is invalid.

```
byte 0: left earbud   — bit 7 = charging, bits 0–6 = percentage (0–100)
byte 1: right earbud  — bit 7 = charging, bits 0–6 = percentage (0–100)
byte 2: case          — bit 7 = charging, bits 0–6 = percentage (0–100)
```

### Case Byte

On the HT08, the case byte consistently reads `0x00` — likely unsupported on this model. `BatteryStatus::case` is still parsed and kept in the data model for other QCY models that might report it, but the GUI's Home tab only renders left/right gauges; the CLI's `battery` command prints all three.

---

## Related

Firmware version uses the sibling characteristic `00000007-...` with the same direct-read pattern — see [version.md](./version.md).
