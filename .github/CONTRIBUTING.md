# Contributing to QCYx

Thanks for your interest in contributing. QCYx is a community project and any help is genuinely appreciated — whether that's code, protocol research, testing on hardware I don't have, or just improving the docs.

---

## Ways to Contribute

- **Protocol research** — Capturing Bluetooth Low Energy (BLE) traffic for unsupported devices or undocumented operations
- **Testing** — Running QCYx on different hardware, operating systems, or device firmware versions and reporting results
- **Code** — CLI commands, GUI, device abstractions, bug fixes
- **Documentation** — Protocol docs, usage guides, translations

---

## Before You Start

For anything beyond a small fix, **open an issue first**. This avoids duplicate work and allows for alignment on approach before time is invested in a PR.

For protocol research specifically, check [`docs/protocol/`](./docs/protocol/README.md) to see what's already documented.

---

## Development Setup

```bash
git clone https://github.com/allansomensi/qcyx
cd qcyx
cargo build
```

Run tests:

```bash
cargo test
```

Run Just for format before submitting:

```bash
just lint-fix
```

If any changes are made to `Cargo.toml`, run `dist generate` before submitting to ensure the Windows build doesn't break:

```bash
dist generate
```

---

## Protocol Research

If you're capturing Bluetooth Low Energy (BLE) traffic to document new operations:

- Capture traffic from the official QCY Android app performing the operation you want to document.
- Cross-reference with the existing docs in [`docs/protocol/`](./docs/protocol/README.md) to understand the session structure.
- Open an issue with your findings before writing a PR — raw captures are a great starting point even without a full write-up.

All reverse engineering must remain **strictly black-box**: observing Bluetooth communication only. Do not decompile, disassemble, or otherwise reverse engineer any QCY binary or proprietary SDK.

---

## Pull Requests

- Keep PRs focused — one feature or fix per PR.
- Reference the related issue in the PR description.
- Make sure `cargo test` and `just lint-fix` pass.
- Update or add documentation if your change affects behavior or the protocol.

---

## Commit Style

Commits follow the [Conventional Commits](https://www.conventionalcommits.org/) convention. Optionally, semantic emojis can be added.

```
feat: add device rename command
fix: handle stale BLE connection state on reconnect
docs: document ANC packet layout
chore: update dependencies
```

---

## Legal

By contributing, you agree that your contributions will be licensed under the [MIT License](./LICENSE).

Do not submit any code derived from QCY's proprietary software, firmware, or SDKs.
