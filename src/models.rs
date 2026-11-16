use crate::domain::Month;
#[derive(Clone,Debug)]
pub struct Settings {pub opening:i64,pub reserve:i64,pub theme:String,pub start:Month,pub last_seen:Month}
#[derive(Clone,Debug)]
pub struct Category {pub id:i64,pub name:String}
#[derive(Clone,Debug)]
pub struct Expense {pub id:i64,pub description:String,pub amount:i64,pub category_id:i64,pub category:String,pub date:String,pub automatic:bool}
#[derive(Clone,Debug)]
pub struct NewCommitment {pub description:String,pub category_id:i64,pub amount:i64,pub installments:Option<u32>,pub start:Month,pub end:Option<Month>}
#[derive(Clone,Debug)]
pub struct Commitment {pub id:i64,pub description:String,pub category_id:i64,pub amount:i64,pub installments:Option<u32>,pub start:Month,pub end:Option<Month>,pub cancelled:Option<Month>}
#[derive(Clone,Debug,Default)]
pub struct MonthlyTotal {pub month:u32,pub income:i64,pub expenses:i64}
#[derive(Clone,Debug,Default)]
pub struct Snapshot {pub balance:i64,pub available:i64,pub income:i64,pub expenses:i64,pub categories:Vec<(String,i64)>,pub annual:Vec<MonthlyTotal>,pub opening_year:i64,pub closing_year:i64}
