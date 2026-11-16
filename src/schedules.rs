use crate::{Result, domain::*, invalid, models::*, store::Store};
use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
pub(crate) fn commitment_rows(conn: &Connection) -> Result<Vec<Commitment>> {
    Ok(conn.prepare("SELECT id,description,category_id,amount,installments,start,end,cancelled FROM commitments ORDER BY start,id")?.query_map([],|r|Ok(Commitment{id:r.get(0)?,description:r.get(1)?,category_id:r.get(2)?,amount:r.get(3)?,installments:r.get(4)?,start:Month(r.get(5)?),end:r.get::<_,Option<i32>>(6)?.map(Month),cancelled:r.get::<_,Option<i32>>(7)?.map(Month)}))?.collect::<std::result::Result<_,_>>()?)
}
pub(crate) fn charge(conn: &Connection, c: &Commitment, m: Month) -> Result<Option<i64>> {
    if m < c.start || c.end.is_some_and(|end| m > end) || c.cancelled.is_some_and(|end| m >= end) {
        return Ok(None);
    }
    if let Some(count) = c.installments {
        let index = (m.0 - c.start.0) as u32;
        if index >= count {
            return Ok(None);
        }
        return installment(c.amount, count, index).map(Some);
    }
    Ok(Some(conn.query_row("SELECT amount FROM commitment_revisions WHERE commitment_id=?1 AND month<=?2 ORDER BY month DESC LIMIT 1",params![c.id,m.0],|r|r.get(0)).optional()?.unwrap_or(c.amount)))
}
pub(crate) fn reconcile_connection(conn: &Connection, today: NaiveDate) -> Result<()> {
    let now = Month::from_date(today).validate()?;
    let start = conn
        .query_row("SELECT start FROM settings WHERE id=1", [], |r| {
            r.get::<_, i32>(0)
        })
        .optional()?;
    let Some(start) = start else { return Ok(()) };
    let commitments = commitment_rows(conn)?;
    for serial in start..=now.0 {
        let m = Month(serial);
        let date = m.first().to_string();
        if let Some(amount) = conn
            .query_row(
                "SELECT amount FROM income_schedules WHERE month<=?1 ORDER BY month DESC LIMIT 1",
                [serial],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        {
            conn.execute("INSERT INTO transactions(kind,description,amount,date,occurrence) VALUES('income','Renda mensal',?1,?2,?3) ON CONFLICT(occurrence) DO NOTHING",params![amount,date,format!("income:{m}")])?;
        }
        for c in &commitments {
            if let Some(amount) = charge(conn, c, m)? {
                let description = if let Some(n) = c.installments {
                    format!("{} ({}/{n})", c.description, serial - c.start.0 + 1)
                } else {
                    c.description.clone()
                };
                conn.execute("INSERT INTO transactions(kind,description,amount,category_id,date,occurrence,commitment_id) VALUES('expense',?1,?2,?3,?4,?5,?6) ON CONFLICT(occurrence) DO NOTHING",params![description,amount,c.category_id,date,format!("commit:{}:{m}",c.id),c.id])?;
            }
        }
    }
    conn.execute(
        "UPDATE settings SET last_seen=MAX(last_seen,?1) WHERE id=1",
        [now.0],
    )?;
    Ok(())
}
impl Store {
    pub fn reconcile(&mut self, today: NaiveDate) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        reconcile_connection(&tx, today)?;
        tx.commit()?;
        Ok(())
    }
    fn schedule_date(&self, today: NaiveDate) -> Result<Month> {
        let now = Month::from_date(today).validate()?;
        let settings = self
            .settings()?
            .ok_or_else(|| invalid("Configure sua renda primeiro."))?;
        if now < settings.last_seen {
            return Err(invalid(
                "O relógio está anterior ao histórico. Corrija a data do computador antes de alterar compromissos.",
            ));
        }
        Ok(now)
    }
    pub fn schedule_income(&mut self, amount: i64, first: Month, today: NaiveDate) -> Result<()> {
        validate_amount(amount, true)?;
        first.validate()?;
        let now = self.schedule_date(today)?;
        if first <= now {
            return Err(invalid(
                "A nova renda deve começar a partir do próximo mês.",
            ));
        }
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        reconcile_connection(&tx, today)?;
        tx.execute("INSERT INTO income_schedules VALUES(?1,?2) ON CONFLICT(month) DO UPDATE SET amount=excluded.amount",params![first.0,amount])?;
        tx.commit()?;
        Ok(())
    }
    pub fn commitments(&self) -> Result<Vec<Commitment>> {
        commitment_rows(&self.conn)
    }
    pub fn charge_at(&self, c: &Commitment, m: Month) -> Result<Option<i64>> {
        charge(&self.conn, c, m)
    }
    pub fn add_commitment(&mut self, c: NewCommitment, today: NaiveDate) -> Result<i64> {
        let name = label(&c.description, 145)?;
        validate_amount(c.amount, false)?;
        let now = self.schedule_date(today)?;
        c.start.validate()?;
        if c.start < now || c.start > now.next() {
            return Err(invalid(
                "O compromisso deve começar neste mês ou no próximo.",
            ));
        }
        if let Some(end) = c.end {
            end.validate()?;
            if end < c.start {
                return Err(invalid("O término deve ser posterior ao início."));
            }
        }
        if let Some(count) = c.installments {
            installment(c.amount, count, 0)?;
            Month(c.start.0 + count as i32 - 1).validate()?;
        }
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO commitments(description,category_id,amount,installments,start,end) VALUES(?1,?2,?3,?4,?5,?6)",params![name,c.category_id,c.amount,c.installments,c.start.0,c.end.map(|m|m.0)])?;
        let id = tx.last_insert_rowid();
        reconcile_connection(&tx, today)?;
        tx.commit()?;
        Ok(id)
    }
    pub fn revise_commitment(&mut self, id: i64, amount: i64, today: NaiveDate) -> Result<()> {
        validate_amount(amount, false)?;
        let next = self.schedule_date(today)?.next().validate()?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM commitments WHERE id=?1 AND installments IS NULL AND (cancelled IS NULL OR cancelled>?2) AND (end IS NULL OR end>=?2))",params![id,next.0],|r|r.get(0))?;
        if !valid {
            return Err(invalid("Apenas recorrências ativas podem ser alteradas."));
        }
        reconcile_connection(&tx, today)?;
        tx.execute("INSERT INTO commitment_revisions VALUES(?1,?2,?3) ON CONFLICT(commitment_id,month) DO UPDATE SET amount=excluded.amount",params![id,next.0,amount])?;
        tx.commit()?;
        Ok(())
    }
    pub fn cancel_commitment(&mut self, id: i64, today: NaiveDate) -> Result<()> {
        let next = self.schedule_date(today)?.next().validate()?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        reconcile_connection(&tx, today)?;
        tx.execute(
            "UPDATE commitments SET cancelled=MIN(COALESCE(cancelled,?1),?1) WHERE id=?2",
            params![next.0, id],
        )?;
        tx.commit()?;
        Ok(())
    }
}
