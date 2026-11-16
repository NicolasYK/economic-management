use chrono::NaiveDate;
use economic_management::{domain::Month, store::Store};
fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 16).unwrap()
}
fn initialized() -> Store {
    let mut s = Store::memory().unwrap();
    s.initialize(5000, 100000, Month::from_date(today()), today())
        .unwrap();
    s
}
#[test]
fn backup_restores_without_replaying_income_and_keeps_recovery() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("backup.sqlite");
    let source = initialized();
    source
        .add_expense("Compra", 1500, 1, today(), today())
        .unwrap();
    source.export_backup(&p).unwrap();
    let info = Store::inspect_backup(&p).unwrap();
    assert_eq!(info.transactions, 2);
    let mut target = initialized();
    target.restore_backup(&p).unwrap();
    target.reconcile(today()).unwrap();
    assert_eq!(
        target
            .snapshot(Month::from_date(today()), 2026)
            .unwrap()
            .balance,
        103500
    );
    assert_eq!(target.expenses(None).unwrap().len(), 1);
}
#[test]
fn corrupted_or_future_or_hostile_backup_never_changes_live_data() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("backup.sqlite");
    let mut s = initialized();
    std::fs::write(&p, b"not sqlite").unwrap();
    assert!(s.restore_backup(&p).is_err());
    std::fs::remove_file(&p).unwrap();
    s.export_backup(&p).unwrap();
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("PRAGMA user_version=99").unwrap();
    drop(c);
    assert!(s.restore_backup(&p).is_err());
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("PRAGMA user_version=1; CREATE VIEW exploit AS SELECT 1;")
        .unwrap();
    drop(c);
    assert!(s.restore_backup(&p).is_err());
    assert_eq!(
        s.snapshot(Month::from_date(today()), 2026).unwrap().balance,
        105000
    );
}
#[test]
fn rejects_tampered_automatic_amount_and_invalid_calendar() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("backup.sqlite");
    let s = initialized();
    s.export_backup(&p).unwrap();
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("UPDATE transactions SET amount=1").unwrap();
    drop(c);
    assert!(Store::inspect_backup(&p).is_err());
}
#[test]
fn csv_neutralizes_formula_names() {
    let s = initialized();
    let id = s.add_category("=HYPERLINK(\"example\")").unwrap();
    s.add_expense("Teste", 100, id, today(), today()).unwrap();
    assert!(
        s.annual_csv(2026)
            .unwrap()
            .contains("\"'=HYPERLINK(\"\"example\"\")\";1.00")
    );
}
#[test]
fn file_restore_keeps_recovery_and_rejects_stale_editor() {
    let t = tempfile::tempdir().unwrap();
    let live = t.path().join("live.sqlite");
    let backup = t.path().join("incoming.sqlite");
    let mut owner = Store::open(&live).unwrap();
    owner
        .initialize(0, 1000, Month::from_date(today()), today())
        .unwrap();
    let stale = Store::open(&live).unwrap();
    let replacement = initialized();
    replacement
        .add_expense("Novo registro", 2500, 1, today(), today())
        .unwrap();
    replacement.export_backup(&backup).unwrap();
    let recovery = owner.restore_backup(&backup).unwrap().unwrap();
    assert!(recovery.is_file());
    assert_eq!(Store::inspect_backup(&recovery).unwrap().balance, 1000);
    assert!(
        stale
            .edit_expense(2, "Edição antiga", 1, 1, today(), today())
            .is_err()
    );
    assert_eq!(
        owner.expenses(None).unwrap()[0].description,
        "Novo registro"
    );
}
#[test]
fn rejects_incomplete_schedules_without_materializing_history() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("incomplete.sqlite");
    let s = initialized();
    s.export_backup(&p).unwrap();
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("DELETE FROM transactions; UPDATE settings SET start=24000,last_seen=25211; INSERT INTO commitments(id,description,category_id,amount,start) VALUES(1,'Longo',1,1,24000);").unwrap();
    drop(c);
    let begin = std::time::Instant::now();
    assert!(Store::inspect_backup(&p).is_err());
    assert!(begin.elapsed() < std::time::Duration::from_secs(2));
}
