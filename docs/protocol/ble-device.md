# BLE Device Identification

QCY earbuds are accessed over BLE, a separate radio link from the classic-Bluetooth (BR/EDR) audio connection. Audio can work with no BLE link, or vice versa — see [error-recovery.md](./error-recovery.md#windows-ble-pairing-vs-classic-bluetooth-audio-pairing).

---

## Discovery: Manufacturer Data

QCY devices are identified by BLE advertisement manufacturer data, not by name.

| Field | Value |
|---|---|
| Manufacturer (Company ID) | `0x521C` |

```
for each advertisement:
    if manufacturer_data contains key 0x521C:
        this is a QCY device
```

No address or name filtering is required.

### Model Identification

Model identification uses the first two bytes of the QCY manufacturer payload
as a big-endian vendor ID: `0x4D48` (AilyBuds Pro) or `0x4D4C` (HT10ANC APP).
These identifiers are listed in the [public product catalog](https://github.com/hui1601/Quicky/blob/main/internal/product/products.json).
When those bytes are unavailable, exact case-insensitive names are accepted:
`QCY AilyBuds Pro`, `QCY HT10` (optionally suffixed `-APP`), and `QCY HT10ANC APP`.
An available vendor ID takes precedence over an editable device name. AilyBuds
Pro+ is identified separately and must not be matched as AilyBuds Pro (HT10).

---

## GATT Service

| Item | Value |
|---|---|
| Service UUID | `0000a001-0000-1000-8000-00805f9b34fb` |

The only service this crate talks to. The device exposes other services as well (see collision warning below), but no confirmed protocol behavior lives outside `0xa001`.

---

## GATT Characteristics

| Characteristic | UUID | Properties | Purpose |
|---|---|---|---|
| Command | `00001001-...` | `WRITE_WITHOUT_RESPONSE` | Outgoing commands |
| Notify | `00001002-...` | `NOTIFY` | Incoming responses/events (echoes, `ANC_RESULT`, ...) |
| Battery | `00000008-...` | `READ`, `NOTIFY` | Battery status — not part of the `0xFF`-framed protocol, see [battery.md](./battery.md) |

Command and Notify carry the `0xFF`-framed format described in [frame-format.md](./frame-format.md).

### ⚠️ Multi-Service UUID Collision

The device exposes a second GATT service (`0x7033`) that reuses the same 16-bit characteristic numbers `0x1001`/`0x1002`. A characteristic lookup that matches by UUID alone can silently resolve to the wrong service — writes and subscriptions succeed at the BLE layer but nothing meaningful happens, with no error anywhere.

**Every characteristic lookup must filter by both `characteristic.uuid` and `characteristic.service_uuid == SERVICE_UUID`.** `btleplug::api::Characteristic` exposes `service_uuid` for exactly this. See [error-recovery.md](./error-recovery.md#multi-service-uuid-collision).

---

## Write Type

Writes to the command characteristic use `WriteType::WithoutResponse` (ATT opcode `0x52`) — no ATT Write Response ever follows. Falls back to `WithResponse` if `WRITE_WITHOUT_RESPONSE` isn't advertised, though on the HT08 this never triggers.

---

## Capability Restrictions

### AilyBuds Pro (HT10): Transparency

The AilyBuds Pro does not support Transparency. Attempts to activate it have
been reported to disconnect or power down the earbuds. For devices matched by
[model identification](#model-identification), QCYx therefore rejects every
Transparency `ANC_SETTING` write before sending it, including CLI commands and
settings applied through profiles. The GUI shows the option disabled with an
unsupported-feature message. Normal and Adaptive ANC remain available. AilyBuds
Pro+ is a distinct variant and is not covered by this restriction.

Battery, firmware `4.0.8` (both earbuds), Normal and Adaptive ANC have been
validated on an AilyBuds Pro over macOS CoreBluetooth, including control while
Mac audio plays through the earbuds. Other controls have not been validated on
this model.
