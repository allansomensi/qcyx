# Channel Balance

Stereo balance between left and right earbud.

---

## Wire Format

```
Write:  ff 03 16 01 <value>
Read:   16 01 <value>          (one block inside the state-sync blob)
```

| Item | Value |
|---|---|
| Opcode | `0x16` |
| Params | 1B: `value` (`0..=100`), `50` = centered, `0` = full left, `100` = full right |

`crate::balance::set_balance` clamps to `0..=100` rather than letting an out-of-range byte reach the device.

---

## Confirmation

Unconfirmed whether a write gets any echo/result, unlike ANC (see [anc.md](./anc.md#confirmation-flow)). `DeviceHandle::set_balance` fires the write and returns without waiting — likely the normal shape for most commands in this protocol, with ANC's two-stage confirmation being the exception.

---

## Notes

Balance has no dedicated connect-time read — `DeviceHandle::read_state_sync` performs one GATT read of the notify characteristic and extracts both the ANC and balance blocks from that single response.
