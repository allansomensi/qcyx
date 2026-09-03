# Device Actions: Reset, Factory Reset, Rename

Three parameterless-or-nearly-so writes, none with a known confirmation/echo mechanism.

---

## Wire Format

```
Reset to default:  ff 02 01 00
Factory reset:      ff 02 03 00
Rename:              ff <2+len> 18 <len> <utf8 name bytes>
```

| Action | Opcode | Params |
|---|---|---|
| Reset to default | `0x01` | None |
| Factory reset | `0x03` | None |
| Rename | `0x18` | Length-prefixed UTF-8 name |

Reset and factory-reset have zero-length params — the trailing `00` is the param-length byte, not a value.

⚠️ Write side confirmed by source/framing only, not directly wire-verified. `0x18`'s *read* side is confirmed (see [version.md](./version.md#the-renameable-pairing-name-0x18)), which raises confidence in the write since both share framing, but the write itself is unverified.

---

## Safety

Factory reset (and, to a lesser extent, reset-to-default) are destructive and irreversible from this crate's side — no undo command exists. Neither `factory_reset` nor `reset_default` confirm on their own; that's the caller's responsibility:

- CLI (`qcyx-cli factory-reset`) prompts `y/N` unless `--yes` is passed.
- GUI Settings tab uses an "arm, then confirm" two-press pattern on the button (`crates/gui/src/view/tabs/settings.rs`).

## Rename Doesn't Update the Cached Device Name

`DeviceHandle::device_name()` returns the BLE advertisement's `local_name` captured at connect time (see [version.md](./version.md#device-name-not-wire-protocol)) — not re-read after a rename write. GUI Home/Settings tabs keep showing the old name until the next reconnect. Whether the device updates its advertised name immediately or only after a reboot/re-pair is unconfirmed.
