use crate::{Result,domain::*,models::*,store::Store};
use rusqlite::params;
impl Store {
    pub fn snapshot(&self,month:Month,year:i32)->Result<Snapshot>{
        month.validate()?;Month(year*12).validate()?;
        let settings=self.settings()?;let opening=settings.as_ref().map_or(0,|s|s.opening);
        let net:i64=self.conn.query_row("SELECT COALESCE(SUM(CASE kind WHEN 'income' THEN amount ELSE -amount END),0) FROM transactions",[],|r|r.get(0))?;
        let (income,expenses)=self.totals(month)?;
        let categories=self.conn.prepare("SELECT c.name,SUM(t.amount) FROM transactions t JOIN categories c ON c.id=t.category_id WHERE t.kind='expense' AND t.date LIKE ?1 GROUP BY c.id ORDER BY SUM(t.amount) DESC,c.name")?.query_map([format!("{month}-%")],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<std::result::Result<_,_>>()?;
        let mut annual=Vec::new();for m in 0..12{let (income,expenses)=self.totals(Month(year*12+m))?;annual.push(MonthlyTotal{month:(m+1) as u32,income,expenses});}
        let before:i64=self.conn.query_row("SELECT COALESCE(SUM(CASE kind WHEN 'income' THEN amount ELSE -amount END),0) FROM transactions WHERE date<?1",[format!("{year:04}-01-01")],|r|r.get(0))?;
        let opening_year=if settings.as_ref().is_some_and(|s|s.start.0/12>year){0}else{opening+before};
        let closing_year=opening_year+annual.iter().map(|m|m.income-m.expenses).sum::<i64>();
        Ok(Snapshot{balance:opening+net,available:opening+net-settings.map_or(0,|s|s.reserve),income,expenses,categories,annual,opening_year,closing_year})
    }
    fn totals(&self,m:Month)->Result<(i64,i64)>{Ok(self.conn.query_row("SELECT COALESCE(SUM(CASE WHEN kind='income' THEN amount ELSE 0 END),0),COALESCE(SUM(CASE WHEN kind='expense' THEN amount ELSE 0 END),0) FROM transactions WHERE date LIKE ?1",[format!("{m}-%")],|r|Ok((r.get(0)?,r.get(1)?)))?)}
    pub fn annual_categories(&self,year:i32)->Result<Vec<(String,i64)>>{
        Ok(self.conn.prepare("SELECT c.name,SUM(t.amount) FROM transactions t JOIN categories c ON c.id=t.category_id WHERE t.kind='expense' AND t.date>=?1 AND t.date<?2 GROUP BY c.id ORDER BY SUM(t.amount) DESC")?.query_map(params![format!("{year}-01-01"),format!("{}-01-01",year+1)],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn annual_csv(&self,year:i32)->Result<String>{
        let s=self.snapshot(Month(year*12),year)?;
        let mut csv=String::from("\u{feff}Mês;Receitas;Despesas;Resultado\r\n");
        for m in s.annual {csv.push_str(&format!("{year}-{:02};{};{};{}\r\n",m.month,input_money(m.income),input_money(m.expenses),input_money(m.income-m.expenses)));}
        csv.push_str("\r\nCategoria;Despesas\r\n");
        for (name,value) in self.annual_categories(year)? {let safe=if name.starts_with(['=','+','-','@','\t','\r']){format!("'{name}")}else{name};csv.push_str(&format!("\"{}\";{}\r\n",safe.replace('"',"\"\""),input_money(value)));}Ok(csv)
    }
}
