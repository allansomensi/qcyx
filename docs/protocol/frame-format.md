# Frame Format

Every command write and every notification uses the same `0xFF`-framed binary format (`qcyx_core::protocol::Command`).

---

## Layout

```
┌──────┬────────────┬────────┬────────────┬─────────────────┐
│ 0xFF │ BodyLength  │ Opcode │ ParamLength│ Parameters...    │
│ 1B   │ 1B          │ 1B     │ 1B         │ ParamLength bytes│
└──────┴────────────┴────────┴────────────┴─────────────────┘
```

| Field | Size | Description |
|---|---|---|
| Start-of-frame | 1B | Always `0xFF` |
| `BodyLength` | 1B | Length of everything after this field (`Opcode` + `ParamLength` + `Parameters`) — always `2 + ParamLength`, validated on parse |
| `Opcode` | 1B | Command/event identifier — see [packet-reference.md](./packet-reference.md) |
| `ParamLength` | 1B | Number of parameter bytes |
| `Parameters` | `ParamLength` bytes | Opcode-specific payload |

### Example: ANC "Normal" write

```
FF 05 17 03 02 00 00
│  │  │  │  └┴┴─ params: mode=0x02, sub_scene=0x00, noise_value=0x00
│  │  │  └─ param length = 3
│  │  └─ opcode = 0x17 (ANC_SETTING)
│  └─ body length = 5
└─ start of frame
```

---

## Multiple Commands Per Packet

A single write or notification can carry multiple command blocks back-to-back after the shared header:

```
FF <body_len> <opcode_1> <len_1> <params_1...> <opcode_2> <len_2> <params_2...> ...
```

`Command::parse` returns a `Vec<Command>`, consuming blocks until the frame ends. A truncated trailing block is a parse error, not silently dropped.

---

## Parse Validation

A packet is rejected if:

* Fewer than 4 bytes total, or the first byte isn't `0xFF`.
* `BodyLength + 2` doesn't match total packet length.
* A block's `ParamLength` would read past the end of the packet.
* The packet parses to zero command blocks.

A parse failure is non-fatal to the connection — the notification is discarded and the stream keeps being watched (see [transport.md](./transport.md)).
