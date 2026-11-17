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

## Remaining verification in progress

- Cross-platform GitHub Actions runs and packaged artifact smoke tests.
- Idle resource measurement.
- Independent review follow-up.

## Release follow-up (2026-11-16, America/Sao_Paulo)

- Fresh WSL run: all 20 tests, formatting and strict Clippy passed.
- GitHub CI run `44005673108`: tests, formatting and Clippy passed on all
  three platforms; security audit passed. Linux graphical launch failed because
  `libxkbcommon-x11.so` was absent. Added the runtime dependency to both workflows
  and the AppImage, with explicit failure if any required library is missing.
- Rebuilt the Linux release binary and AppImage; launching the extracted AppImage
  under Xvfb completed successfully (exit 0). WSL Mesa warnings remain nonfatal.
- Independent source review and follow-up review of release changes found no
  blocking correctness/security defects. Native package CI remains the publication gate.
- Main protection configured with required PR, up-to-date Linux/Windows/macOS and
  security checks, administrator enforcement, resolved conversations, no force push
  and no deletion. This solo-maintainer workflow requires zero external approvals.

This ledger distinguishes observed checks from pending work; workflow configuration
alone is not evidence that a platform build or a release was published.
