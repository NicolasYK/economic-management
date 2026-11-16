use chrono::NaiveDate;
use economic_management::{domain::Month, models::NewCommitment, store::Store};
fn date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}
fn month(s: &str) -> Month {
    s.parse().unwrap()
}
fn setup() -> Store {
    let mut s = Store::memory().unwrap();
    s.initialize(10000, 300000, month("2026-11"), date("2026-11-16"))
        .unwrap();
    s
}
fn plan(total: i64, count: Option<u32>) -> NewCommitment {
    NewCommitment {
        description: "Teste".into(),
        category_id: 1,
        amount: total,
        installments: count,
        start: month("2026-11"),
        end: None,
    }
}
#[test]
fn catches_up_once_and_reserve_is_not_an_expense() {
    let mut s = setup();
    s.add_expense("Mercado", 12345, 1, date("2026-11-16"), date("2026-11-16"))
        .unwrap();
    s.set_preferences(50000, "dark").unwrap();
    s.reconcile(date("2027-01-01")).unwrap();
    s.reconcile(date("2027-01-01")).unwrap();
    let report = s.snapshot(month("2026-11"), 2026).unwrap();
    assert_eq!(report.balance, 897655);
    assert_eq!(report.available, 847655);
    assert_eq!(report.expenses, 12345);
    assert_eq!(report.income, 300000);
    assert_eq!(report.annual.len(), 12);
    assert_eq!(report.annual[10].expenses, 12345);
}
#[test]
fn installment_stops_at_exact_total_and_income_changes_only_in_future() {
    let mut s = setup();
    s.add_commitment(plan(10000, Some(3)), date("2026-11-16"))
        .unwrap();
    s.schedule_income(400000, month("2026-12"), date("2026-11-16"))
        .unwrap();
    s.reconcile(date("2027-04-01")).unwrap();
    let rows = s.expenses(None).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows.iter().map(|r| r.amount).sum::<i64>(), 10000);
    assert_eq!(s.snapshot(month("2026-11"), 2026).unwrap().income, 300000);
    assert_eq!(s.snapshot(month("2026-12"), 2026).unwrap().income, 400000);
    s.reconcile(date("2026-11-16")).unwrap();
    assert!(s.clock_reversed(date("2026-11-16")).unwrap());
    assert_eq!(s.expenses(None).unwrap().len(), 3);
}
#[test]
fn recurrence_revisions_and_cancel_preserve_past() {
    let mut s = setup();
    let id = s
        .add_commitment(plan(9000, None), date("2026-11-16"))
        .unwrap();
    s.revise_commitment(id, 10000, date("2026-11-16")).unwrap();
    s.reconcile(date("2026-12-01")).unwrap();
    s.cancel_commitment(id, date("2026-12-15")).unwrap();
    s.reconcile(date("2027-02-01")).unwrap();
    let rows = s.expenses(None).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows.iter().map(|r| r.amount).sum::<i64>(), 19000);
    assert!(s.delete_expense(rows[0].id).is_err());
}
#[test]
fn manual_crud_validates_dates_and_categories() {
    let s = setup();
    let today = date("2026-11-16");
    assert!(s.add_expense("", 100, 1, today, today).is_err());
    assert!(
        s.add_expense("Future", 100, 1, date("2026-12-01"), today)
            .is_err()
    );
    assert!(
        s.add_expense("Past", 100, 1, date("2026-10-01"), today)
            .is_err()
    );
    assert!(
        s.add_expense("Bad category", 100, 900, today, today)
            .is_err()
    );
    let category = s
        .add_category("Viagens '); DROP TABLE categories;--")
        .unwrap();
    let id = s
        .add_expense("Ônibus", 400000, category, today, today)
        .unwrap();
    assert!(s.snapshot(month("2026-11"), 2026).unwrap().balance < 0);
    s.edit_expense(id, "Ônibus", 1000, category, today, today)
        .unwrap();
    assert!(s.delete_category(category).is_err());
    s.rename_category(category, "Viagens").unwrap();
    s.delete_expense(id).unwrap();
    s.delete_category(category).unwrap();
    assert!(s.expenses(None).unwrap().is_empty());
}
#[test]
fn concurrent_reconciliation_and_reopen_are_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("finance.sqlite");
    let mut s = Store::open(&path).unwrap();
    s.initialize(0, 10000, month("2026-11"), date("2026-11-01"))
        .unwrap();
    drop(s);
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let p = path.clone();
            std::thread::spawn(move || {
                Store::open(&p)
                    .unwrap()
                    .reconcile(date("2027-02-01"))
                    .unwrap();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(
        Store::open(&path)
            .unwrap()
            .snapshot(month("2027-02"), 2027)
            .unwrap()
            .balance,
        40000
    );
}
