# mouse-noti

Desktop notification mouse indicator. Shows a visual cue at the cursor when a system notification fires on macOS or Windows.

## Status

Phase 1 (MVP). Indicator only. Radial menu planned for v2.

## Build

Prereqs: pnpm 9, Rust (stable), Tauri build deps for your OS.

```bash
pnpm install
pnpm tauri dev                       # dev mode
pnpm tauri dev --features mock-os    # dev mode + inject_mock_event command
pnpm tauri build                     # release bundle (DMG / MSI)
```

## Permissions

- **macOS**: Accessibility (System Settings → Privacy & Security → Accessibility). First launch prompts.
- **Windows**: Notification listener consent (first-run prompt).

## Tests

```bash
pnpm test:run                                              # frontend (vitest)
cargo test --manifest-path src-tauri/Cargo.toml --lib      # rust units
```

## Docs

- Spec: `docs/superpowers/specs/2026-05-13-mouse-noti-design.md`
- Plan: `docs/superpowers/plans/2026-05-13-mouse-noti-mvp.md`
- Manual QA: `docs/superpowers/checklists/mvp-manual-qa.md`
