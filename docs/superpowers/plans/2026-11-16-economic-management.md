# Economic Management Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement task by task.

**Goal:** Deliver the approved offline desktop finance application and tested release packages.
**Architecture:** Rust domain services own money and calendar rules; SQLite owns durable transactions; egui views call services. Backup imports reconstruct validated records rather than executing imported SQL.
**Tech Stack:** Rust, egui/eframe, rusqlite, chrono, native file dialogs, GitHub Actions.
**Spec:** `docs/superpowers/specs/2026-11-16-economic-management-design.md`, approved by the user on 2026-11-16.

## Global constraints

- Offline runtime, BRL integer cents, Portuguese UI, all six screens.
- No cloud, account, telemetry or background daemon.
- Income once per month; catch up while closed; preserve historical schedule amounts.
- Reserve affects available balance, not ledger balance.
- SQLite backups, 50 MiB input cap, reconstruction and atomic recovery.
- SemVer + commit in UI; bilingual README and changelog; Windows/Linux releases and macOS build.

## Review focus

- Concurrent reconciliation must not duplicate money (task 2).
- Invalid or malicious backups must not modify the current ledger (task 3).
- Clock reversal and schedule editing must preserve history (task 2).
- Financial rounding and limits must not silently lose cents (task 1).
- A restored database must preserve occurrence identities when reopened (task 3).

## Task 1: Money and calendar domain

Files: `Cargo.toml`, `rust-toolchain.toml`, `src/lib.rs`, `src/domain.rs`, `tests/domain.rs`.
Interfaces: `parse_money(&str) -> Result<i64>`, `money(i64) -> String`, `Month(i32)` with parsing/display/addition; `installment(total, count, index) -> Result<i64>`.
- [ ] Write tests for comma decimals, invalid fractions, negative opening balance, overflow, 10000/3 = [3334,3333,3333], and December rollover.
- [ ] Run `cargo test --test domain`; verify failure before implementation.
- [ ] Implement bounded integer arithmetic and calendar helpers.
- [ ] Run full tests, fmt and clippy; commit with title and body.

## Task 2: SQLite ledger and recurring finance

Files: `src/store.rs`, `src/schema.sql`, `src/models.rs`, `src/schedules.rs`, `tests/ledger.rs`.
Interfaces: `Store::open(path)`, `initialize(opening,income,first_month,today)`, `reconcile(today)`, manual expense CRUD, category CRUD, income scheduling, recurring/instalment creation and cancellation, `snapshot(month,year)`.
- [ ] Write tests for initial balance, monthly catch-up, repeated/concurrent reconciliation, terminal installments, future revisions, negative balance and twelve-month reports.
- [ ] Run ledger tests and observe missing behavior.
- [ ] Implement schema constraints, immediate transactions and parameterized queries; inject today's date as a parameter.
- [ ] Verify tests and commit the ledger and schedule slices separately when coherent.

## Task 3: Portable backup and CSV

Files: `src/backup.rs`, `src/report.rs`, `tests/backup.rs`.
Interfaces: `Store::export_backup(path)`, `inspect_backup(path)`, `Store::restore_backup(path)`; `annual_csv(year)`.
- [ ] Test round trip, malformed/future schema, hostile schema, oversized input, invalid values, failed restore preserving data and repeated reconcile after restore.
- [ ] Observe failures; implement read-only validation and canonical reconstruction, then transactional replacement and automatic recovery backup.
- [ ] Test CSV quoting/formula neutralization; run suite and commit.

## Task 4: Desktop UI

Files: `src/main.rs`, `src/ui/mod.rs`, `src/ui/dashboard.rs`, `src/ui/forms.rs`, `src/ui/pages.rs`, `src/ui/style.rs`, `build.rs`.
Interfaces: `FinanceApp::new(Store)`; views operate on store services and cached snapshots refreshed after mutations or date changes.
- [ ] Add headless egui smoke tests for empty/onboarded screens and theme selection.
- [ ] Implement onboarding, six navigation areas, expense and commitment dialogs, category management, donut chart, annual report, backup dialogs and confirmations.
- [ ] Test each screen, keyboard paths, invalid input retention, theme changes, long lists and version rendering; commit.

## Task 5: Distribution, docs and security review

Files: `.github/workflows/{ci,release}.yml`, `.github/PULL_REQUEST_TEMPLATE.md`, `scripts/*`, `README.md`, `CHANGELOG.md`, `SECURITY.md`, `docs/verification.md`.
- [ ] Build release executable; run unit/integration/UI tests, fmt, clippy and dependency audit.
- [ ] Measure startup and idle CPU/RAM with 10,000 synthetic expenses; record actual environment and results.
- [ ] Package Windows ZIP, Linux AppImage, macOS ZIP; verify version/tag synchronization and checksums.
- [ ] Request independent whole-branch review, address confirmed findings and rerun affected checks.
- [ ] Commit with meaningful bodies; publish GitHub release only for verified artifacts and report any external blockers honestly.

## Execution ledger

- Preflight: existing checkout is clean on codex/technical-design; implement in this branch as continuation of approved design, preserving main.
- User approved design and explicitly requested implementation; no second approval of this plan is needed.
- Interfaces: tasks 2–4 share Store; domain validation is canonical for both UI and backup.
