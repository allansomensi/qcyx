# QCY BLE Protocol

Reverse-engineered application-level protocol layered on top of BLE GATT, used to communicate with QCY earbuds. Baseline model is the **Melobuds Pro**; other QCY models are expected to share framing and service structure, but scene tables and opcodes should be re-verified per model.

All values marked ✅ **Confirmed** were validated against real wire traffic. Values marked ⚠️ are hypothesized (source/reference only) and not yet wire-confirmed. Third-party reference repos (`hui1601/Quicky`, `HttpKiwi/OpenQCY`, `pedro-labsabs/521C`, `GiorgioRafael/OpenQCY-Desktop`) are used only for orientation — several of their values (e.g. opcode `0x0C` for ANC) proved wrong against real traffic, so nothing is trusted without independent confirmation.

---

## Document Index

### Foundation

| Document | Description |
|---|---|
| [ble-device.md](./ble-device.md) | Advertising, manufacturer ID, GATT service/characteristics |
| [transport.md](./transport.md) | GATT write/notify pattern, timeouts |
| [frame-format.md](./frame-format.md) | The `0xFF`-framed binary command/response format |
| [error-recovery.md](./error-recovery.md) | Error handling, retry strategy, Windows quirks |

### Operations

| Document | Description |
|---|---|
| [anc.md](./anc.md) | ANC scene table, opcodes, write/confirm flow |
| [balance.md](./balance.md) | Channel balance opcode and wire format |
| [battery.md](./battery.md) | Battery status characteristic |
| [version.md](./version.md) | Firmware version, device name, pairing name |
| [eq.md](./eq.md) | Equalizer presets |
| [device-actions.md](./device-actions.md) | Reset, factory reset, rename |

### Reference

| Document | Description |
|---|---|
| [packet-reference.md](./packet-reference.md) | Master opcode table |
| [implementation-notes.md](./implementation-notes.md) | Libraries, portability checklist, open questions |

---

## Session Flow

```
SCAN        → find peripheral advertising QCY manufacturer data
CONNECT     → BLE connect, with liveness retry
DISCOVER    → find the 0xa001 service, with retry
SUBSCRIBE   → subscribe to every NOTIFY characteristic in the service
OPERATE     → write a command, await echo + async confirmation
DISCONNECT  → CLI: after every command · GUI: on exit / link loss
```

This is a **write + notify** transport, not request/response: every command is a `WriteWithoutResponse` GATT write, and the device replies asynchronously on the notify characteristic — sometimes with more than one notification per write (see [anc.md](./anc.md#confirmation-flow)).

---

## CLI vs. GUI Connection Lifecycle

* **CLI**: connects, sends one command, awaits confirmation, disconnects. Stateless; pays scan+connect+discovery cost per invocation.
* **GUI**: connects once at startup and keeps the link open; commands reuse the connection, a background task watches for disconnects.

Both share the same protocol implementation — only connection lifecycle differs.
