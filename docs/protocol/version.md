# Firmware Version

Same direct-read pattern as [battery.md](./battery.md).

---

## Characteristic

| Item | Value |
|---|---|
| UUID | `00000007-0000-1000-8000-00805f9b34fb` |
| Service | `0000a001-...` |

Plain GATT characteristic, read directly — no opcode, no `Command` framing.

⚠️ Not yet wire-confirmed: byte layout below is a hypothesis mirroring the `major.minor.patch` shape seen elsewhere in the protocol.

## Payload Layout

```
bytes 0–2: left earbud version  — major.minor.patch
bytes 3–5: right earbud version — major.minor.patch
```

A 3-byte payload still parses (left version only). An empty payload is invalid.

---

## Device Name (Not Wire Protocol)

`btleplug::PeripheralProperties::local_name` already exposes the BLE advertisement name — the same one shown in OS Bluetooth settings. `DeviceHandle::device_name` just surfaces it; no dedicated GATT read exists for this.

## The Renameable Pairing Name (`0x18`)

`CMDID_PAIRNAME` (`0x18`) is a separate, renameable pairing name, distinct from the advertisement name. Writing it is implemented — see [device-actions.md](./device-actions.md). Reading it is confirmed but not yet wired in.

### Reading

Uses the same `0xFE`-request mechanism as the connect-time state-sync read (see [anc.md](./anc.md#reading-the-current-scene-at-connect-time)): the host writes `ff 03 fe 01 18` (request CMDID `0x18`), and the device replies on notify with an `0x18` block:

```
ff 22 18 20 51 43 59 20 4d 65 6c 6f 42 75 64 73 20 50 72 6f 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

`param_len` is fixed at `0x20` (32), null-padded — not length-prefixed by content. Decoding as UTF-8 up to the first `0x00` gives `"QCY MeloBuds Pro"`.

Not part of the state-sync blob — requested separately. Implementing the read would mean either a dedicated `0xFE`-triggered function or extending `read_state_sync`, following the ANC/balance pattern.
