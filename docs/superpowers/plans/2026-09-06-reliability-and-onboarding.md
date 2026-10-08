# Reliability and onboarding implementation plan

Goal: preserve settings, enforce shell confirmation, report action failures, restore tests, and provide first-run guidance and safe previews.

The user approved this work following the project review, then requested commit and deployment. Keep the existing settings UI edits and release through the repository's tag-triggered workflow.

## Design

Use a typed preference patch that cannot replace menus or hotkeys. Serialize every settings read-modify-write in SettingsStore. Write configuration atomically with a recoverable backup. Show save errors and allow retry.

Resolve actions on the backend and show a native confirmation for shell actions marked confirm. Execute the exact confirmed snapshot off the UI thread. Check exit status and include bounded stderr in errors. Retain the existing error badge/log pipeline.

Add a dismissible getting-started panel, automatically show settings until onboarding is completed, and keep preview controls available afterward. Test notifications work without OS notification permissions. Menu preview uses an inline settings component with no execution IPC, isolated from the executable radial window. Reuse current visual styles. Broader profiles, import/export, mixed-DPI, and notification-history changes remain future work.

## Tasks

- [x] Settings: add regression coverage for preference patches preserving menus/hotkeys, concurrent edits, and recoverable writes; implement store mutation API and convert commands. Convert Settings UI to patches with ordered saves and visible errors.
- [x] Actions: add nonzero-exit coverage; enforce native shell confirmation and execute off UI thread; retain bounded diagnostics; repair Windows URL opening using the existing opener integration.
- [x] Tests: restrict Vitest discovery to src; update outdated UI expectations and selected-state accessibility; cover settings saves and preview safety.
- [x] Onboarding: add persisted completion preference, startup settings visibility, test-notification command, safe inline radial preview, guide and permanent preview controls.
- [x] Verify: run frontend tests, TypeScript/Vite build, Rust unit tests (default and mock-os), inspect final diff, document manual OS checks and any limitations.

## Acceptance

Adding a menu/hotkey then changing appearance preserves both immediately and after reload. Concurrent mutations do not lose entries. A declined shell prompt never executes a process. Nonzero exit reports an error; long commands do not block the UI. Previews do not launch commands, apps, or URLs. Onboarding can be dismissed and does not return after restart. Existing user modifications remain intact.

## Verification and release

Frontend: 15 passing tests and production Vite build. Rust: 30 passing unit tests, plus mock-os binary/doc test targets. Independent code review found no actionable correctness/security issues. The initial 0.6.7 Windows CI caught an unused platform import under `-D warnings`; the import is now conditional, and version metadata is aligned to 0.6.8 without rewriting the published tag.

Manual OS checks remain: native confirmation visibility/cancellation, Windows app/URL launch, and physical multi-monitor indicator placement. These have not been verified by automated tests. Existing macOS mixed-DPI and notification-history limitations are outside this release's scope.
