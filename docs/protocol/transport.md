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
| Scan duration | `5s` | Time to scan for QCY advertisements |
| ANC confirmation timeout | `10s` | Covers echo + `ANC_RESULT`; real-world latency is 2.5–3.1s on a clean connection, with generous margin for Windows |
| Post-write flush delay | `300ms` | Lets the platform BLE stack flush before a caller can disconnect |

---

## Byte Order

Every confirmed field so far (opcode, param length, mode, sub-scene, noise value, `ANC_RESULT` status) is a single byte. No multi-byte integer field exists yet, so no byte-order convention has been established.
