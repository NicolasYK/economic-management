use economic_management::{domain::Month, store::Store};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("Pass a dedicated empty QA directory")?,
    );
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("finance.sqlite");
    if path.exists() {
        return Err("QA database already exists; choose a new directory".into());
    }
    let today = chrono::Local::now().date_naive();
    let month = Month::from_date(today);
    let mut s = Store::open(&path)?;
    s.initialize(10000000, 850000, month, today)?;
    s.set_preferences(2000000, "light")?;
    s.add_category("Moradia")?;
    s.add_category("Transporte")?;
    s.add_category("Saúde")?;
    drop(s);
    let mut c = rusqlite::Connection::open(&path)?;
    let tx = c.transaction()?;
    {
        let mut statement=tx.prepare("INSERT INTO transactions(kind,description,amount,category_id,date) VALUES('expense',?1,?2,?3,?4)")?;
        for i in 0..10000 {
            statement.execute(rusqlite::params![
                format!("Despesa de exemplo {i}"),
                100 + i % 3000,
                1 + i % 5,
                today.to_string()
            ])?;
        }
    }
    tx.commit()?;
    drop(c);
    let start = Instant::now();
    let mut s = Store::open(&path)?;
    s.reconcile(today)?;
    let snapshot = s.snapshot(month, month.0 / 12)?;
    let rows = s.expenses(None)?;
    println!(
        "records={} open_reconcile_report_ms={:.2} database_bytes={} balance_cents={}",
        rows.len(),
        start.elapsed().as_secs_f64() * 1000.0,
        std::fs::metadata(path)?.len(),
        snapshot.balance
    );
    Ok(())
}
