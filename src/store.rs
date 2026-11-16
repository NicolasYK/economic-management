use crate::{Result,invalid,domain::*,models::*};
use chrono::NaiveDate;
use rusqlite::{Connection,OptionalExtension,params};
use std::{path::{Path,PathBuf},time::Duration};
pub(crate) const SCHEMA:&str=include_str!("schema.sql");
pub struct Store {pub(crate) conn:Connection,pub(crate) path:Option<PathBuf>}
impl Store {
    pub fn memory()->Result<Self>{Self::connect(Connection::open_in_memory()?,None)}
    pub fn open(path:&Path)->Result<Self>{
        if let Some(parent)=path.parent().filter(|p|!p.as_os_str().is_empty()){std::fs::create_dir_all(parent)?;}
        Self::connect(Connection::open(path)?,Some(path.to_owned()))
    }
    fn connect(mut conn:Connection,path:Option<PathBuf>)->Result<Self>{
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF;")?;
        let tx=conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let version:i64=tx.pragma_query_value(None,"user_version",|r|r.get(0))?;
        let tables:i64=tx.query_row("SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",[],|r|r.get(0))?;
        if version==0 && tables==0 {tx.execute_batch(SCHEMA)?;}
        else if version!=1 || tx.pragma_query_value(None,"application_id",|r|r.get::<_,i64>(0))?!=1162692401 {return Err(invalid("Banco incompatível. Use uma versão compatível do aplicativo."));}
        tx.commit()?; Ok(Self{conn,path})
    }
    pub fn path(&self)->Option<&Path>{self.path.as_deref()}
    pub fn settings(&self)->Result<Option<Settings>>{
        Ok(self.conn.query_row("SELECT opening,reserve,theme,start,last_seen FROM settings WHERE id=1",[],|r|Ok(Settings{opening:r.get(0)?,reserve:r.get(1)?,theme:r.get(2)?,start:Month(r.get(3)?),last_seen:Month(r.get(4)?)})).optional()?)
    }
    pub fn initialize(&mut self,opening:i64,income:i64,first:Month,today:NaiveDate)->Result<()> {
        validate_amount(opening.checked_abs().ok_or_else(||invalid("Saldo inválido."))?,true)?; validate_amount(income,true)?;
        let now=Month::from_date(today).validate()?; first.validate()?;
        if first<now || first>now.next(){return Err(invalid("O primeiro crédito deve ser neste mês ou no próximo."));}
        let tx=self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if tx.query_row("SELECT count(*) FROM settings",[],|r|r.get::<_,i64>(0))?>0{return Err(invalid("O controle financeiro já foi iniciado."));}
        tx.execute("INSERT INTO settings(id,opening,start,last_seen) VALUES(1,?1,?2,?2)",params![opening,now.0])?;
        tx.execute("INSERT INTO income_schedules VALUES(?1,?2)",params![first.0,income])?;
        crate::schedules::reconcile_connection(&tx,today)?;tx.commit()?;Ok(())
    }
    pub fn set_preferences(&self,reserve:i64,theme:&str)->Result<()> {
        validate_amount(reserve,true)?;
        if !["light","dark","system"].contains(&theme){return Err(invalid("Tema inválido."));}
        self.conn.execute("UPDATE settings SET reserve=?1,theme=?2 WHERE id=1",params![reserve,theme])?;Ok(())
    }
    pub fn categories(&self)->Result<Vec<Category>> {
        Ok(self.conn.prepare("SELECT id,name FROM categories ORDER BY name")?.query_map([],|r|Ok(Category{id:r.get(0)?,name:r.get(1)?}))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn add_category(&self,name:&str)->Result<i64>{
        let name=label(name,80)?;
        self.conn.execute("INSERT INTO categories(name,normalized) VALUES(?1,?2)",params![name,name.to_lowercase()])?;Ok(self.conn.last_insert_rowid())
    }
    pub fn rename_category(&self,id:i64,name:&str)->Result<()> {
        let name=label(name,80)?; self.conn.execute("UPDATE categories SET name=?1,normalized=?2 WHERE id=?3",params![name,name.to_lowercase(),id])?;Ok(())
    }
    pub fn delete_category(&self,id:i64)->Result<()>{self.conn.execute("DELETE FROM categories WHERE id=?1",[id])?;Ok(())}
    fn expense_input(&self,description:&str,amount:i64,date:NaiveDate,today:NaiveDate)->Result<String>{
        validate_amount(amount,false)?;Month::from_date(date).validate()?;
        let settings=self.settings()?.ok_or_else(||invalid("Configure sua renda primeiro."))?;
        if date>today || date<settings.start.first(){return Err(invalid("A data deve estar entre o início do controle e hoje."));}
        label(description,160)
    }
    pub fn add_expense(&self,description:&str,amount:i64,category:i64,date:NaiveDate,today:NaiveDate)->Result<i64>{
        let description=self.expense_input(description,amount,date,today)?;
        self.conn.execute("INSERT INTO transactions(kind,description,amount,category_id,date) VALUES('expense',?1,?2,?3,?4)",params![description,amount,category,date.to_string()])?;Ok(self.conn.last_insert_rowid())
    }
    pub fn edit_expense(&self,id:i64,description:&str,amount:i64,category:i64,date:NaiveDate,today:NaiveDate)->Result<()> {
        let description=self.expense_input(description,amount,date,today)?;
        let n=self.conn.execute("UPDATE transactions SET description=?1,amount=?2,category_id=?3,date=?4 WHERE id=?5 AND kind='expense' AND occurrence IS NULL",params![description,amount,category,date.to_string(),id])?;
        if n==0{return Err(invalid("Somente despesas manuais podem ser editadas."));}Ok(())
    }
    pub fn delete_expense(&self,id:i64)->Result<()> {
        if self.conn.execute("DELETE FROM transactions WHERE id=?1 AND kind='expense' AND occurrence IS NULL",[id])?==0{return Err(invalid("Somente despesas manuais podem ser excluídas."));}Ok(())
    }
    pub fn expenses(&self,month:Option<Month>)->Result<Vec<Expense>>{
        let prefix=month.map(|m|format!("{m}-%"));
        Ok(self.conn.prepare("SELECT t.id,t.description,t.amount,c.id,c.name,t.date,t.occurrence IS NOT NULL FROM transactions t JOIN categories c ON c.id=t.category_id WHERE t.kind='expense' AND (?1 IS NULL OR t.date LIKE ?1) ORDER BY c.name,t.date DESC,t.id DESC")?.query_map([prefix],|r|Ok(Expense{id:r.get(0)?,description:r.get(1)?,amount:r.get(2)?,category_id:r.get(3)?,category:r.get(4)?,date:r.get(5)?,automatic:r.get(6)?}))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn income_at(&self,month:Month)->Result<i64>{Ok(self.conn.query_row("SELECT amount FROM income_schedules WHERE month<=?1 ORDER BY month DESC LIMIT 1",[month.0],|r|r.get(0)).optional()?.unwrap_or(0))}
    pub fn clock_reversed(&self,today:NaiveDate)->Result<bool>{Ok(self.settings()?.is_some_and(|s|s.last_seen>Month::from_date(today)))}
}
