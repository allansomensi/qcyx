cli-welcome = Starting QCYx Bluetooth Manager...

core-connecting = Connecting to device...
core-connected = Connected!
core-connected-named = Connected to {$name}!
core-subscribed = Subscribed to the notification channel!

error-bluetooth-adapter = Could not find a suitable Bluetooth adapter.
error-device-not-found = No QCY devices found. Make sure they are out of the case.
error-service-not-found = QCY GATT service not found on device.
error-connection-dropped = Connected, then the device immediately dropped the link. This usually means it isn't BLE-paired with Windows yet — go to Settings > Bluetooth & devices and pair it there (this is separate from the audio pairing you already did), then try again.
error-operation-timeout = The Bluetooth stack stopped responding, so the operation was abandoned. Reconnect and try again.

cli-anc-set = ANC scene set — mode: {$mode}, sub-scene: {$sub_scene}, noise value: {$noise_value}
cli-anc-unconfirmed = ANC command sent, but the device did not confirm it — it may not have applied.
cli-anc-timeout = ANC command sent, but no confirmation arrived in time — it may not have applied.
cli-anc-echoed = ANC scene accepted by the device (echo received); waiting for final confirmation...

cli-battery-title = Battery status:
battery-left = Left earbud
battery-right = Right earbud
battery-case = Case
battery-charging = charging

cli-version-title = Device info:
cli-version-name-label = Name
cli-version-name-unknown = Unknown
cli-version-unknown = Unknown

cli-balance-set = Channel balance set to {$value} (0 = full left, 100 = full right, 50 = centered).

cli-eq-set = Equalizer preset applied.

cli-reset-default-done = Settings reset to default.
cli-factory-reset-confirm = This will erase all settings on the device and cannot be undone. Continue?
cli-factory-reset-cancelled = Factory reset cancelled.
cli-factory-reset-done = Factory reset sent.
cli-rename-done = Device renamed to "{$name}". The new name will show up after reconnecting.
cli-rename-empty = The name was empty once whitespace and control characters were removed — nothing was sent to the device.

cli-notification-volume-set = Notification volume set.
cli-scheduled-power-off-set = Scheduled power-off timer set.
cli-disconnect-power-off-set = Disconnect power-off timer set.
cli-wear-detection-set = Wear detection set.
cli-game-mode-set = Game mode set.
cli-sleep-mode-set = Sleep mode set.
cli-ldac-set = LDAC toggle set.
cli-multipoint-set = Multipoint (dual-device) toggle set.
cli-touch-action-set = Touch action set.

waiting-title = Waiting for device...
waiting-subtitle = Make sure your earbuds are nearby and out of the case.
connected-header = Connected
error-title = Connection Error
error-unknown = Unknown error

gui-anc-title = Noise Control
gui-anc-normal = Normal
gui-anc-transparency = Transparency
gui-anc-hint = Choose a mode to apply it to your earbuds.
gui-anc-applied = ANC scene applied.
gui-anc-unconfirmed = ANC command sent, but the device did not confirm it — it may not have applied.
gui-anc-timeout = ANC command sent, but no confirmation arrived in time — it may not have applied.
gui-anc-echoed = ANC scene accepted by the device (echo received); waiting for final confirmation...
gui-anc-error = Failed to set ANC scene: {$error}
gui-balance-error = Failed to set channel balance: {$error}
gui-retry = Retry

# Sidebar
app-title = QCYx
app-subtitle = QCY earbuds controller
nav-home = Home
nav-anc = Noise Control
nav-equalizer = Equalizer
nav-settings = Settings
nav-about = About
theme-label = Theme
badge-connecting = Connecting
badge-connected = Connected
badge-error = Error

# Shared
badge-coming-soon = Coming soon

# Home
home-device-fallback-name = QCY HT08
home-firmware-label = Firmware:
home-firmware-unknown = Unknown
home-battery-title = Battery
home-battery-left = Left Earbud
home-battery-right = Right Earbud
home-battery-case = Case
home-battery-stale = Last reading may be outdated
home-quick-actions-title = Quick actions
home-action-find-device = Find my earbuds
home-action-game-mode = Game mode
home-action-on = On
home-action-off = Off
home-status-ready = Ready
home-status-idle = Idle

# ANC tab
anc-title = Noise Control
anc-subtitle = Choose a mode to apply it to your earbuds.
anc-noise-cancelling = Noise Cancelling
anc-active = Active

anc-transparency-detail-title = Transparency
anc-vocal-enhancement-label = Vocal Enhancement
anc-vocal-enhancement-desc = Prioritizes voices in ambient sound. Turn off for an adjustable ambient level instead.
anc-ambient-level-label = Ambient sound level

anc-nc-title = Noise Cancelling
anc-nc-adaptive = Adaptive
anc-nc-wind = Wind Resistance
anc-nc-indoor = Indoor
anc-nc-daily-commute = Daily Commute
anc-nc-noisy = Noisy

anc-balance-title = Channel Balance
anc-balance-centered = Centered
anc-balance-right = {$percent}% right
anc-balance-left = {$percent}% left
balance-reset-button = Reset

# Equalizer tab
eq-title = Equalizer
eq-subtitle = Fine-tune your earbuds' sound.
eq-preset-label = Preset
eq-hint = Pick a preset to apply it to your earbuds.
eq-preset-spatial = Spatial Audio
eq-preset-default = Default
eq-preset-popular = Popular
eq-preset-bass = Bass Boost
eq-preset-rock = Rock
eq-preset-soft = Soft
eq-preset-classic = Classic
eq-preset-applied = Equalizer preset applied.
eq-preset-error = Failed to set equalizer preset: {$error}
eq-custom-title = Custom
eq-custom-desc = Per-band editing isn't supported yet — the device's custom EQ format hasn't been fully reverse-engineered.

# Settings tab
settings-title = Settings
settings-subtitle = Preferences and device information.
settings-device-name-label = Device name
settings-device-name-placeholder = QCY HT08
settings-save-button = Save
settings-rename-done = Device renamed — the new name will show up after reconnecting.
settings-rename-error = Failed to rename device: {$error}
settings-inear-toggle-label = In-ear detection
settings-inear-toggle-desc = Pauses playback when an earbud is removed.
settings-inear-toggle-error = Failed to set wear detection: {$error}
settings-game-mode-label = Game mode
settings-game-mode-desc = Lowers audio latency for gaming.
settings-game-mode-error = Failed to set game mode: {$error}
settings-sleep-mode-label = Sleep mode
settings-sleep-mode-desc = Optimizes the earbuds for sleeping.
settings-sleep-mode-error = Failed to set sleep mode: {$error}
settings-ldac-label = LDAC codec
settings-ldac-desc = Higher-quality Bluetooth audio codec, when supported by the source.
settings-ldac-error = Failed to set LDAC: {$error}
settings-multipoint-label = Dual-device connection
settings-multipoint-desc = Connects to two devices at once and switches audio between them.
settings-multipoint-error = Failed to set multipoint: {$error}
settings-touch-action-title = Touch actions
settings-touch-action-desc = Assigns what each tap gesture does, per earbud.
settings-touch-action-error = Failed to set touch action: {$error}
settings-touch-action-left-single = Left · Single tap
settings-touch-action-right-single = Right · Single tap
settings-touch-action-left-double = Left · Double tap
settings-touch-action-right-double = Right · Double tap
settings-touch-action-left-triple = Left · Triple tap
settings-touch-action-right-triple = Right · Triple tap
settings-touch-action-none = No action
settings-touch-action-play-pause = Play / Pause
settings-touch-action-previous = Previous track
settings-touch-action-next = Next track
settings-touch-action-voice-assistant = Voice assistant
settings-touch-action-volume-up = Volume up
settings-touch-action-volume-down = Volume down
settings-touch-action-game-mode = Game mode
settings-touch-action-anc = ANC
settings-notification-volume-title = Notification volume
settings-notification-volume-low = Low
settings-notification-volume-medium = Medium
settings-notification-volume-high = High
settings-notification-volume-max = Max
settings-notification-volume-error = Failed to set notification volume: {$error}
settings-scheduled-poweroff-title = Scheduled power-off
settings-scheduled-poweroff-desc = Powers off after a set time, regardless of playback or connection state.
settings-scheduled-poweroff-off = Off
settings-scheduled-poweroff-custom-placeholder = Custom (minutes)
settings-scheduled-poweroff-custom-button = Set
settings-scheduled-poweroff-error = Failed to set scheduled power-off: {$error}
settings-disconnect-poweroff-title = Power off after disconnect
settings-disconnect-poweroff-desc = Powers off this many minutes after losing the Bluetooth connection.
settings-disconnect-poweroff-never = Never
settings-disconnect-poweroff-error = Failed to set disconnect power-off: {$error}
settings-minutes-format = {$minutes} min
settings-firmware-section-title = Firmware
settings-firmware-version-label = Installed version
settings-firmware-check-button = Check for updates
settings-reset-default-desc = Resets all settings on the device to their default values.
settings-reset-default-button = Reset to default
settings-reset-default-done = Settings reset to default.
settings-reset-default-error = Failed to reset settings: {$error}
settings-confirm-again = Tap again to confirm
settings-danger-title = Danger zone
settings-factory-reset-desc = Resets your earbuds to factory settings.
settings-factory-reset-button = Factory reset
settings-factory-reset-done = Factory reset sent.
settings-factory-reset-error = Factory reset failed: {$error}

# About tab
about-description = QCYx is an unofficial, open-source tool for controlling QCY Bluetooth earbuds over BLE/GATT, built from reverse-engineering the protocol.
about-repo-label = Repository
about-license-label = License
about-protocol-title = About the protocol
about-protocol-note = The protocol was mapped by directly capturing BLE traffic from the official app. Only capture-confirmed commands are implemented; the rest of this interface is already in place, waiting on protocol confirmation.
