use crate::{Result, domain::*, invalid, store::Store};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params, types::Value};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
const TABLES: [&str; 6] = [
    "settings",
    "income_schedules",
    "categories",
    "commitments",
    "commitment_revisions",
    "transactions",
];
pub struct BackupInfo {
    pub transactions: i64,
    pub categories: i64,
    pub balance: i64,
}

fn schema(conn: &Connection) -> Result<Vec<(String, String, String)>> {
    Ok(conn
        .prepare(
            "SELECT type,name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY name",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<std::result::Result<_, _>>()?)
}
// Names are exclusively from TABLES. Only bound scalar values cross the trust boundary.
fn copy_records(source: &Connection, target: &Connection) -> Result<()> {
    for table in TABLES.iter().rev() {
        target.execute(&format!("DELETE FROM {table}"), [])?;
    }
    for table in TABLES {
        let mut select = source.prepare(&format!("SELECT * FROM {table}"))?;
        let columns = select.column_count();
        let placeholders = vec!["?"; columns].join(",");
        let mut insert = target.prepare(&format!("INSERT INTO {table} VALUES({placeholders})"))?;
        let mut rows = select.query([])?;
        while let Some(row) = rows.next()? {
            let values = (0..columns)
                .map(|i| row.get::<_, Value>(i))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            insert.execute(rusqlite::params_from_iter(values))?;
        }
    }
    Ok(())
}
fn canonical_backup(path: &Path) -> Result<Store> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > 50 * 1024 * 1024 {
        return Err(invalid(
            "O backup deve ser um arquivo SQLite de até 50 MiB.",
        ));
    }
    let source = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    source.busy_timeout(Duration::from_secs(2))?;
    source.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA query_only=ON; BEGIN;")?;
    let version: i64 = source.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let application: i64 = source.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if version != 1 || application != 1162692401 {
        return Err(invalid("Versão de backup incompatível."));
    }
    let mut clean = Store::memory()?;
    if schema(&source)? != schema(&clean.conn)? {
        return Err(invalid(
            "A estrutura do backup não corresponde ao aplicativo.",
        ));
    }
    let integrity: String = source.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err(invalid("O backup está corrompido."));
    }
    for table in TABLES {
        let n: i64 =
            source.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?;
        if n > 100_000 {
            return Err(invalid(
                "O backup excede o limite de 100 mil registros por tabela.",
            ));
        }
    }
    let tx = clean.conn.transaction()?;
    copy_records(&source, &tx)?;
    tx.commit()?;
    validate(&mut clean)?;
    Ok(clean)
}
fn validate(s: &mut Store) -> Result<()> {
    let settings = s
        .settings()?
        .ok_or_else(|| invalid("O backup ainda não possui um controle financeiro configurado."))?;
    for c in s.categories()? {
        if c.id <= 0 || label(&c.name, 80)? != c.name {
            return Err(invalid("Categoria inválida."));
        }
        let normalized: String = s.conn.query_row(
            "SELECT normalized FROM categories WHERE id=?1",
            [c.id],
            |r| r.get(0),
        )?;
        if normalized != c.name.to_lowercase() {
            return Err(invalid("Categoria inconsistente."));
        }
    }
    for c in s.commitments()? {
        if c.id <= 0 || label(&c.description, 145)? != c.description || c.start < settings.start {
            return Err(invalid("Compromisso inválido."));
        }
        if let Some(n) = c.installments {
            installment(c.amount, n, 0)?;
            Month(c.start.0 + n as i32 - 1).validate()?;
        }
    }
    let bad_schedules:i64=s.conn.query_row("SELECT (SELECT count(*) FROM income_schedules WHERE month<?1)+(SELECT count(*) FROM commitment_revisions r JOIN commitments c ON c.id=r.commitment_id WHERE r.month<c.start OR c.installments IS NOT NULL)",[settings.start.0],|r|r.get(0))?;
    if bad_schedules != 0 {
        return Err(invalid("Vigência de renda ou recorrência inválida."));
    }
    let commitments = s.commitments()?;
    let by_id: std::collections::HashMap<_, _> = commitments.iter().map(|c| (c.id, c)).collect();
    let mut query=s.conn.prepare("SELECT id,kind,description,amount,date,occurrence,commitment_id,category_id FROM transactions")?;
    let mut rows = query.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let kind: String = row.get(1)?;
        let name: String = row.get(2)?;
        let amount: i64 = row.get(3)?;
        let date_string: String = row.get(4)?;
        let date = parse_date(&date_string)?;
        let m = Month::from_date(date);
        if id <= 0
            || date.to_string() != date_string
            || m < settings.start
            || m > settings.last_seen
            || label(&name, 160)? != name
        {
            return Err(invalid("Lançamento inválido no backup."));
        }
        let key: Option<String> = row.get(5)?;
        let origin: Option<i64> = row.get(6)?;
        if let Some(key) = key {
            if date != m.first() {
                return Err(invalid("Data de ocorrência automática inválida."));
            }
            if kind == "income" {
                let expected=s.conn.query_row("SELECT amount FROM income_schedules WHERE month<=?1 ORDER BY month DESC LIMIT 1",[m.0],|r|r.get::<_,i64>(0)).optional()?;
                if key != format!("income:{m}") || expected != Some(amount) {
                    return Err(invalid("Renda automática inconsistente."));
                }
            } else {
                let c = origin
                    .and_then(|id| by_id.get(&id))
                    .ok_or_else(|| invalid("Origem de parcela ausente."))?;
                if key != format!("commit:{}:{m}", c.id)
                    || crate::schedules::charge(&s.conn, c, m)? != Some(amount)
                    || row.get::<_, i64>(7)? != c.category_id
                {
                    return Err(invalid("Parcela ou recorrência inconsistente."));
                }
            }
        }
    }
    drop(rows);
    drop(query);
    let first_income: Option<i32> =
        s.conn
            .query_row("SELECT MIN(month) FROM income_schedules", [], |r| r.get(0))?;
    let expected = first_income.map_or(0, |first| (settings.last_seen.0 - first + 1).max(0) as i64);
    let actual: i64 = s.conn.query_row(
        "SELECT count(*) FROM transactions WHERE kind='income'",
        [],
        |r| r.get(0),
    )?;
    if expected != actual {
        return Err(invalid("O backup possui ocorrências mensais ausentes."));
    }
    let counts:std::collections::HashMap<i64,i64>=s.conn.prepare("SELECT commitment_id,count(*) FROM transactions WHERE commitment_id IS NOT NULL GROUP BY commitment_id")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<std::result::Result<_,_>>()?;
    for c in &commitments {
        let mut last = settings.last_seen.0;
        if let Some(end) = c.end {
            last = last.min(end.0);
        }
        if let Some(end) = c.cancelled {
            last = last.min(end.0 - 1);
        }
        if let Some(n) = c.installments {
            last = last.min(c.start.0 + n as i32 - 1);
        }
        let expected = (last - c.start.0 + 1).max(0) as i64;
        if counts.get(&c.id).copied().unwrap_or(0) != expected {
            return Err(invalid("O backup possui cobranças ausentes."));
        }
    }
    Ok(())
}
impl Store {
    pub fn export_backup(&self, path: &Path) -> Result<()> {
        // create_new refuses existing files, symlinks and the live database itself.
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        drop(file);
        let result = (|| -> Result<()> {
            let mut destination = Connection::open(path)?;
            let backup = rusqlite::backup::Backup::new(&self.conn, &mut destination)?;
            backup.run_to_completion(128, Duration::from_millis(5), None)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(path);
        }
        result
    }
    pub fn inspect_backup(path: &Path) -> Result<BackupInfo> {
        let clean = canonical_backup(path)?;
        let settings = clean.settings()?.unwrap();
        Ok(BackupInfo {
            transactions: clean
                .conn
                .query_row("SELECT count(*) FROM transactions", [], |r| r.get(0))?,
            categories: clean.categories()?.len() as i64,
            balance: clean
                .snapshot(settings.last_seen, settings.last_seen.0 / 12)?
                .balance,
        })
    }
    pub fn restore_backup(&mut self, path: &Path) -> Result<Option<PathBuf>> {
        let clean = canonical_backup(path)?;
        self.replace_backup(clean, || {})
    }
    fn replace_backup(
        &mut self,
        clean: Store,
        after_recovery: impl FnOnce(),
    ) -> Result<Option<PathBuf>> {
        let tx = self.begin_write(rusqlite::TransactionBehavior::Exclusive)?;
        let recovery = if let Some(live) = &self.path {
            let suffix = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| invalid("Relógio inválido."))?
                .as_nanos();
            let p = live.with_file_name(format!("recovery-{suffix}.sqlite"));
            // Capture the locked live snapshot before allowing any other writer.
            let recovery_store = Store::memory()?;
            copy_records(&tx, &recovery_store.conn)?;
            recovery_store.export_backup(&p)?;
            Some(p)
        } else {
            None
        };
        after_recovery();
        // One SQLite transaction replaces rows, preserving the open file identity for other processes.
        // SQLite rolls the entire replacement back if any insertion or commit fails.
        copy_records(&clean.conn, &tx)?;
        tx.execute("UPDATE schema_migrations SET version=?1", params![1])?;
        tx.commit()?;
        Ok(recovery)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation_never_materializes_missing_history() {
        let mut s = Store::memory().unwrap();
        let date = parse_date("2026-11-16").unwrap();
        s.initialize(0, 100, Month::from_date(date), date).unwrap();
        s.conn.execute("DELETE FROM transactions", []).unwrap();
        assert!(validate(&mut s).is_err());
        assert_eq!(
            s.conn
                .query_row("SELECT count(*) FROM transactions", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn recovery_and_replacement_hold_one_exclusive_lock() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("live.sqlite");
        let date = parse_date("2026-11-16").unwrap();
        let mut s = Store::open(&path).unwrap();
        s.initialize(0, 100, Month::from_date(date), date).unwrap();
        let other = Connection::open(&path).unwrap();
        other.busy_timeout(Duration::ZERO).unwrap();
        let mut replacement = Store::memory().unwrap();
        replacement
            .initialize(0, 200, Month::from_date(date), date)
            .unwrap();
        s.replace_backup(replacement, || {
            assert!(
                other
                    .execute("UPDATE settings SET opening=999", [])
                    .is_err(),
                "another connection wrote in the recovery/restore gap"
            );
        })
        .unwrap();
    }
}
