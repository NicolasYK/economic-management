# Verification ledger

## Environment

- Development host: Windows, Ubuntu 24.04 in WSL.
- Rust 1.100.0; locked Cargo dependencies; SQLite bundled into executable.
- Synthetic data only; no personal financial records used for testing.

## Evidence recorded during implementation

- Domain tests initially failed on unimplemented money/instalment functions.
- Ledger tests initially failed because the storage/schedule API did not exist.
- Backup tests initially failed because import/export methods did not exist.
- UI test initially failed because onboarding did not render.
- After implementation and review fixes: 20 automated tests passed; `cargo fmt --check` and strict
  `cargo clippy --all-targets -- -D warnings` passed.
- `cargo audit --json`: zero known vulnerabilities, four informational maintenance
  advisories. Rationale and review deadline are in SECURITY.md.
- Linux release build succeeded. Actual native window smoke test under Xvfb exited
  successfully; WSL Mesa emitted graphics-driver warnings, without application failure.
- Windows native onboarding, first income credit, dashboard and annual report were
  exercised with synthetic values (15,000 opening + 5,000 income = 20,000 balance).
- Regression tests cover an exclusive restore lock, non-mutating backup validation,
  stale edits from another instance, canceled installment progress and editor reset.
- A mutation reversing installment rounding was rejected by the cent-preservation test.
- With 10,000 synthetic expenses: opening, reconciliation and annual report took
  63.12 ms in WSL Ubuntu 24.04; SQLite file size was 921,600 bytes. This measures the
  data path, not complete graphical startup, and is specific to this machine.

## Manual verification limits

- Cross-platform backup transfer and interrupted-process restore recovery were not
  exercised manually; automated restore invariants and native launch are covered.

## Release follow-up (2026-11-16, America/Sao_Paulo)

- Fresh WSL run: all 20 tests, formatting and strict Clippy passed.
- GitHub CI run `44005673108`: tests, formatting and Clippy passed on all
  three platforms; security audit passed. Linux graphical launch failed because
  `libxkbcommon-x11.so` was absent. Added the runtime dependency to both workflows
  and the AppImage, with explicit failure if any required library is missing.
- Rebuilt the Linux release binary and AppImage; launching the extracted AppImage
  under Xvfb completed successfully (exit 0). WSL Mesa warnings remain nonfatal.
- Idle measurement with 10,000 synthetic expenses under WSL/Xvfb: after 5 seconds
  warm-up, a 15-second sample used 0.067% of one CPU core and 177,736 KiB RSS
  (about 174 MiB). This includes this machine's graphics stack and is not a
  cross-platform resource guarantee.
- Independent source review and follow-up review of release changes found no
  blocking correctness/security defects. Native package CI remains the publication gate.
- Main protection configured with required PR, up-to-date Linux/Windows/macOS and
  security checks, administrator enforcement, resolved conversations, no force push
  and no deletion. This solo-maintainer workflow requires zero external approvals.

This ledger distinguishes observed checks from pending work; workflow configuration
alone is not evidence that a platform build or a release was published.

## Windows release correction (2026-11-17)

- All four PR checks and all four main checks passed for the original promotion.
- Release run `44007816252` passed Linux AppImage and macOS ZIP build/launch, but
  Windows ZIP launch timed out. Publication was blocked; no 0.1.0 assets were released.
- Diagnostic CI run `44009392829` reproduced `OpenGL(PainterError("egui_glow
  requires opengl 2.0+. "))` on the Windows hosted runner. Smoke mode now exits
  without an interactive error dialog and captures the native error.
- A regression verifies invalid smoke invocations exit noninteractively; the local
  suite now has 21 passing tests, with formatting and strict Clippy passing.
- Windows CI/package tests use Mesa 25.0.7 MSVC with a pinned SHA-256, copied only
  into the disposable test extraction. The driver is not installed globally and
  is not shipped in the release ZIP. Normal users need an OpenGL 2.0+ driver.
- Version 0.1.1 retains the failed 0.1.0 tag instead of rewriting its history.

## Published release (2026-11-17)

- [PR #2 CI](https://github.com/NicolasYK/economic-management/actions/runs/44010294908)
  and [official main CI](https://github.com/NicolasYK/economic-management/actions/runs/44072897077)
  passed all three platform jobs and the security job, including 21 tests,
  formatting, strict Clippy and Windows packaged graphical launch.
- [Release run](https://github.com/NicolasYK/economic-management/actions/runs/44074293924)
  passed version/branch validation, all three native builds, audits, package launch
  tests and publication. Windows used the isolated Mesa software renderer.
- [v0.1.1](https://github.com/NicolasYK/economic-management/releases/tag/v0.1.1)
  is published as a stable release (not draft/prerelease), from official commit
  `7d79a8c72bc718407b2a456f0f4166b9e9e85f97` on main.
- Verified the three published `.sha256` files against GitHub's uploaded-asset
  SHA-256 digests. Packages are Windows x86_64 ZIP, Linux x86_64 AppImage and
  macOS arm64 ZIP. No 0.1.0 release assets were published.
- Main protection was read back from the API: required PR, all four checks with
  strict freshness, administrators included, resolved conversations, force push
  and deletion disabled. `codex/technical-design` remains the development branch.

| Package | Published SHA-256 |
| --- | --- |
| Linux x86_64 | `dd2d904ea0a4a210cc9207b61fc026f5eabc5c44fb537d6cbba2f90cd6dca6b9` |
| macOS arm64 | `2c914d9529334690e0c7091ce937d155aacf1d98c31fde83410380edf42377d6` |
| Windows x86_64 | `45a2f91bf01200eb9f3fe3fbbd73a15450fb02ae0360555ad0fa9bc3ee1da492` |
