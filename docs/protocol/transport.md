# Transport Layer

Writes go out on the command characteristic; responses arrive asynchronously as notifications on the notify characteristic. There is no guaranteed 1:1 correspondence between a write and a notification.

---

## Subscribing to Notifications

1. **Subscribe to every `NOTIFY`-capable characteristic** in the service, not just `0x1002`. Only `0x1002` is required for ANC, but subscribing to all avoids diverging from the connection pattern the firmware expects. Failure on a secondary characteristic is non-fatal; failure on `0x1002` is fatal.
2. **Unsubscribe immediately before subscribing**, even on a fresh connection, to clear stale CCCD state from a previous session.

---

## Write / Notify Pattern

```
subscribe to notify characteristic (0x1002)
write command to command characteristic (0x1001)   [WriteWithoutResponse]
sleep briefly (flush margin)
watch the notify stream for events relevant to this command
```

See [anc.md](./anc.md#confirmation-flow) for the two-notification example this produces.

---

## Timeouts

| Parameter | Value | Notes |
|---|---|---|
| Scan timeout | `8s` | Upper bound — the scan resolves on the first matching advertisement |
| ANC confirmation timeout | `10s` | Covers echo + `ANC_RESULT`; real-world latency is 2.5–3.1s on a clean connection, with generous margin for Windows |
| Parameter query timeout | `5s` | Reply to a `0xFE` query |
| Post-write flush | `60ms` | After a `WriteWithoutResponse` only, which carries no ATT ack |
| Pre-disconnect flush | `250ms` | One-shot CLI commands that wrote, before the teardown; read-only commands skip it |
| GATT operation budget | `5s` | Every other call into the BLE stack: reads, writes, subscriptions, liveness checks, disconnect |
| Connect budget | `12s` per attempt | See [error-recovery.md](./error-recovery.md) |
| Discovery budget | `8s` per attempt | Service and characteristic discovery |

---

## Byte Order

Multi-byte integer fields are little-endian: the power-off timers (`u16` minutes, see [device-settings.md](./device-settings.md)) and the custom EQ records (`u16` frequency in Hz, `i16` gain in hundredths of a dB, see [eq.md](./eq.md#custom-per-band-curve)). Every other confirmed field is a single byte.
