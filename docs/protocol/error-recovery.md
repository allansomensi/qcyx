# Error Recovery & Platform Quirks

Windows' WinRT BLE stack behaves worse than macOS/Linux for this device; most retry/defensive logic in `qcyx_core` exists to work around it.

---

## Windows BLE Pairing vs. Classic-Bluetooth Audio Pairing

**The single most common "it just doesn't connect" cause.** Audio pairing and BLE/GATT pairing are separate. Windows can let `connect()` return `Ok`, then tear the link down once it finds the app unauthorized for GATT access — btleplug surfaces this as a generic `NotConnected` error, with no hint that pairing is the actual issue.

**Symptom:** connection appears to succeed, then immediately drops. Surfaced as `CoreError::ConnectionDropped`.

**Fix:** pair the device via *Settings → Bluetooth & devices*, separately from audio pairing. This crate can only detect and clearly report the symptom:

```
error-connection-dropped =
  Connected, then the device immediately dropped the link. This usually
  means it isn't BLE-paired with Windows yet — go to Settings > Bluetooth
  & devices and pair it there (this is separate from the audio pairing
  you already did), then try again.
```

The pairing hint is Windows-specific; other platforms show `error-connection-dropped-generic` instead.

---

## Connect Retry

`connect()` succeeding doesn't guarantee the link stays up. `connect_with_retry` connects, waits, then double-checks `is_connected()` before trusting the result.

| Parameter | Value |
|---|---|
| Retries | 3 |
| Post-connect settle delay | 400ms |
| Backoff | `500ms × attempt_number` |

If every retry ends in a dropped link, the final error is `CoreError::ConnectionDropped` — almost always the pairing issue above.

---

## Service Discovery Retry

`discover_services()` can race the WinRT GATT cache and return before it's populated — the connection is real, but the OS-side cache hasn't caught up. `discover_characteristics_with_retry` retries on zero matching characteristics or transient errors.

| Parameter | Value |
|---|---|
| Retries | 3 |
| Backoff | `500ms × attempt_number` |

Not observed on macOS/Linux, but harmless there — only fires when discovery comes back empty.

---

## Multi-Service UUID Collision

See [ble-device.md](./ble-device.md#-multi-service-uuid-collision). The device exposes a second GATT service (`0x7033`) reusing the same characteristic numbers (`0x1001`, `0x1002`) as the target service (`0xa001`).

**Symptom:** connect, discovery, subscribe, and write all succeed at the BLE layer — and nothing happens. No error anywhere.

**Root cause:** characteristic lookups matching by `uuid` alone can resolve to the wrong service.

**Fix:** every lookup filters by both `uuid` and `service_uuid == SERVICE_UUID` (`btleplug::api::Characteristic::service_uuid`).

**General lesson:** never match a characteristic by UUID alone on a device with more than one custom service.

---

## Error Reference

| Error | Meaning | Typical cause |
|---|---|---|
| `AdapterNotFound` | No usable Bluetooth adapter | No hardware, or adapter disabled |
| `DeviceNotFound` | Scan completed without QCY manufacturer data | Earbuds in case, out of range, or not advertising |
| `ServiceNotFound` | Connected, but `0xa001` never appeared after retries | Wrong device, or discovery genuinely failed |
| `CharacteristicNotFound` | Service found, command/notify characteristic missing | Unexpected firmware/model variant |
| `ConnectionDropped` | Connect succeeded, link died before setup finished | Windows BLE pairing/bonding not done |
| `InvalidPacket` | `0xFF`-framed buffer failed to parse | Malformed notification; non-fatal, ignored |
| `BluetoothError` | Passthrough of underlying `btleplug` error | Platform-specific BLE stack failure |
| `NotConnected` | A background poll found no open session | Link lost; the GUI reconnects through `watch_disconnect` |
| `OperationTimeout` | A BLE stack call exceeded its budget | Link died uncleanly; stack stopped responding |
| `FrameTooLarge` | Parameters don't fit the frame's one-byte body length | Programming error — rejected before anything is sent |
| `InvalidName` | A rename sanitizes to an empty name | Blank or control-character-only input |

---

## Connection Lifecycle Cleanup

A BLE peripheral stops advertising while connected. Leaving a failed connection attempt open silently breaks every subsequent scan until the OS times it out. Every failure path in `client::connect` disconnects on error; every successful CLI command funnels through `finish`, which guarantees `disconnect()` regardless of outcome.

The GUI session follows the same principle at a coarser grain: when a command fails at the transport level (`CoreError::is_transport`), the cached connection is dropped and best-effort disconnected, so the next command reconnects from a clean state. Validation and parsing failures leave a healthy link alone — reconnecting would only fail the same way.

### Not Every Read Failure Should Trigger This

Dropping the connection is right for a deliberate write that hit a transport failure, but wrong for a periodic background poll — a single transient read hiccup shouldn't tear down and rebuild the whole session. `read_battery` and `read_rssi` run only against an already-open session: they never connect and never drop it on failure. Detecting an actually-dead link is `is_connected`'s job, checked on its own interval (`watch_disconnect`), and it best-effort disconnects the handle it releases. A poll that connected on its own could also be cancelled mid-connect by a GUI state change, leaving the peripheral connected — and a connected peripheral stops advertising, so the next scan finds nothing.
