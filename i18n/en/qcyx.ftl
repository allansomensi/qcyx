cli-welcome = Starting QCYx Bluetooth Manager...
cli-error = Error: {$error}

core-connecting = Connecting to device...
core-connected = Connected!
core-connected-named = Connected to {$name}!
core-subscribed = Subscribed to the notification channel!

error-bluetooth-adapter = Could not find a suitable Bluetooth adapter.
error-device-not-found = No QCY devices found. Make sure they are out of the case.
error-service-not-found = QCY GATT service not found on device.
error-connection-dropped = Connected, then the device immediately dropped the link. This usually means it isn't BLE-paired with Windows yet — go to Settings > Bluetooth & devices and pair it there (this is separate from the audio pairing you already did), then try again.
error-operation-timeout = The Bluetooth stack stopped responding, so the operation was abandoned. Reconnect and try again.
error-connection-dropped-generic = Connected, then the device immediately dropped the link. Make sure the earbuds are out of the case and within range, then try again.
error-not-connected = Not connected to the device.

cli-anc-set = ANC scene set — mode: {$mode}, sub-scene: {$sub_scene}, noise value: {$noise_value}
cli-anc-unconfirmed = ANC command sent, but the device did not confirm it — it may not have applied.
cli-anc-timeout = ANC command sent, but no confirmation arrived in time — it may not have applied.
cli-anc-echoed = ANC scene accepted by the device (echo received), but the final confirmation didn't arrive in time.

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

cli-eq-custom-set = Custom equalizer curve applied.

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
error-title = Connection Error
error-unknown = Unknown error

gui-anc-normal = Normal
gui-anc-transparency = Transparency
gui-anc-hint = Choose a mode to apply it to your earbuds.
gui-anc-applied = ANC scene applied.
gui-anc-unconfirmed = ANC command sent, but the device did not confirm it — it may not have applied.
gui-anc-timeout = ANC command sent, but no confirmation arrived in time — it may not have applied.
gui-anc-echoed = ANC scene accepted by the device (echo received), but the final confirmation didn't arrive in time.
gui-anc-error = Failed to set ANC scene: {$error}
gui-balance-error = Failed to set channel balance: {$error}
gui-retry = Retry

# Sidebar
app-title = QCYx
app-subtitle = QCY earbuds controller
nav-home = Home
nav-anc = Noise Control
nav-equalizer = Equalizer
nav-profiles = Profiles
nav-settings = Settings
nav-about = About
theme-label = Theme
language-label = Language
badge-connecting = Connecting
badge-connected = Connected
badge-error = Error
sidebar-active-profile = Profile: {$name}

# Shared
badge-coming-soon = Coming soon

# Home
home-device-fallback-name = QCY HT08
home-firmware-label = Firmware:
home-firmware-unknown = Unknown
home-battery-title = Battery
home-battery-left = Left Earbud
home-battery-right = Right Earbud
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
eq-custom-desc = Per-band editing, 31 Hz to 16 kHz, -8 to 8 dB.
eq-custom-reset = Reset
eq-custom-applied = Custom equalizer curve applied.
eq-custom-error = Failed to set custom equalizer curve: {$error}

# Settings tab
settings-title = Settings
settings-subtitle = Preferences and device information.
settings-device-name-label = Device name
settings-device-name-placeholder = QCY HT08
settings-save-button = Save
settings-edit-button = Edit
settings-cancel-button = Cancel
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
settings-firmware-readonly-note = QCYx only reads the firmware version — it never writes firmware to the device. Flashing OTA updates from an unofficial, reverse-engineered client is too risky (a failed write can brick the earbuds), so that capability isn't part of this app.
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
about-author-label = Author
about-license-label = License
about-issues-label = Report an issue
about-firmware-title = Firmware updates
about-firmware-note = QCYx only ever reads the installed firmware version — it never writes firmware to the device. Flashing OTA updates from an unofficial, reverse-engineered client is too risky for a third-party tool to take on, so that capability is intentionally not part of QCYx.
about-disclaimer-title = Disclaimer
about-disclaimer-note = QCYx is an independent, community project with no affiliation to QCY. It was built entirely from observed Bluetooth traffic, without access to QCY's source code. Use it at your own discretion.
about-support-title = Support & feedback
about-support-note = Found a bug or have a feature request? Issues and pull requests are welcome on the repository above.

# Signal strength
signal-tooltip-title = Bluetooth signal strength
signal-tooltip-value = {$rssi} dBm — the closer to 0, the stronger the connection.
signal-tooltip-unknown = Signal strength unavailable on this platform/adapter.

# Profiles
profiles-title = Profiles
profiles-subtitle = Apply a full set of settings in one action, save your own, or import/export them as JSON.
profiles-built-in-title = Built-in
profiles-custom-title = Your profiles
profiles-empty = No saved profiles yet — configure the device the way you like it, then save it below.
profiles-save-current-title = Save current settings as a profile
profiles-name-placeholder = Profile name
profiles-save-button = Save profile
profiles-apply-button = Apply
profiles-active-label = Active
profiles-deactivate-button = Deactivate
profiles-export-button = Export
profiles-delete-button = Delete
profiles-import-button = Import
profiles-applied = Profile applied.
profiles-apply-error = Failed to apply profile: {$error}
profiles-saved = Profile "{$name}" saved.
profiles-export-done = Profile exported.
profiles-export-error = Failed to export profile: {$error}
profiles-import-done = Profile "{$name}" imported.
profiles-import-error = Failed to import profile: {$error}
profile-error-parse = The file is not a valid QCYx profile: {$reason}
profile-error-too-large = The file is too large to be a QCYx profile.
profile-error-name = The profile name is empty or reserved.
profile-error-empty = The profile contains no settings.
profile-error-eq-conflict = The profile sets both an equalizer preset and a custom curve.
profile-error-out-of-range = The profile has an out-of-range value in "{$field}".

built-in-focus = Focus
built-in-calls = Calls
built-in-workout = Workout
built-in-gaming = Gaming

# EQ profiles (Equalizer tab)
eq-profiles-title = EQ profiles
eq-profiles-empty = No saved EQ profiles yet.
eq-profiles-save-button = Save EQ
eq-profiles-applied = EQ profile applied.
eq-profiles-apply-error = Failed to apply EQ profile: {$error}
eq-profiles-saved = EQ profile "{$name}" saved.
eq-profiles-import-done = EQ profile "{$name}" imported.
