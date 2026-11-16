use crate::{Result, invalid};
use chrono::{Datelike, NaiveDate};
use std::{fmt, str::FromStr};
pub const MAX_MONEY: i64 = 1_000_000_000_000;

pub fn parse_money(text: &str) -> Result<i64> {
    let text = text.trim();
    let (negative, text) = text.strip_prefix('-').map_or((false, text), |s| (true, s));
    let parts: Vec<_> = text.split([',', '.']).collect();
    if parts.len() > 2
        || parts[0].is_empty()
        || !parts[0].bytes().all(|c| c.is_ascii_digit())
        || parts[0].len() > 13
    {
        return Err(invalid(
            "Informe um valor sem separador de milhares, como 1234,56.",
        ));
    }
    let fraction = if parts.len() == 2 {
        if parts[1].is_empty()
            || parts[1].len() > 2
            || !parts[1].bytes().all(|c| c.is_ascii_digit())
        {
            return Err(invalid("Use no máximo duas casas decimais."));
        }
        parts[1].parse::<i64>().unwrap() * if parts[1].len() == 1 { 10 } else { 1 }
    } else {
        0
    };
    let value = parts[0]
        .parse::<i64>()
        .map_err(|_| invalid("Valor inválido."))?
        * 100
        + fraction;
    validate_amount(value, true)?;
    Ok(if negative { -value } else { value })
}
pub fn validate_amount(value: i64, zero: bool) -> Result<()> {
    if value < if zero { 0 } else { 1 } || value > MAX_MONEY {
        return Err(invalid(
            "Valor fora do limite permitido (até R$ 10 bilhões).",
        ));
    }
    Ok(())
}
pub fn money(cents: i64) -> String {
    let absolute = cents.unsigned_abs();
    let digits = (absolute / 100).to_string();
    let mut grouped = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push('.');
        }
        grouped.push(ch);
    }
    format!(
        "{}R$ {},{:02}",
        if cents < 0 { "-" } else { "" },
        grouped,
        absolute % 100
    )
}
pub fn input_money(cents: i64) -> String {
    format!(
        "{}{}.{:02}",
        if cents < 0 { "-" } else { "" },
        cents.unsigned_abs() / 100,
        cents.unsigned_abs() % 100
    )
}
pub fn installment(total: i64, count: u32, index: u32) -> Result<i64> {
    validate_amount(total, false)?;
    if !(1..=360).contains(&count) || index >= count || total < i64::from(count) {
        return Err(invalid(
            "Use de 1 a 360 parcelas, com pelo menos um centavo por parcela.",
        ));
    }
    Ok(total / i64::from(count) + i64::from(i64::from(index) < total % i64::from(count)))
}
pub fn label(text: &str, max: usize) -> Result<String> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > max || text.chars().any(char::is_control) {
        return Err(invalid(
            "Preencha o nome dentro do limite de caracteres, sem caracteres de controle.",
        ));
    }
    Ok(text.to_owned())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Month(pub i32);
impl Month {
    pub fn from_date(date: NaiveDate) -> Self {
        Self(date.year() * 12 + date.month0() as i32)
    }
    pub fn first(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.0 / 12, (self.0 % 12 + 1) as u32, 1).expect("validated month")
    }
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
    pub fn validate(self) -> Result<Self> {
        if !(24000..=25211).contains(&self.0) {
            Err(invalid("Use datas entre 2000 e 2100."))
        } else {
            Ok(self)
        }
    }
}
impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.0 / 12, self.0 % 12 + 1)
    }
}
impl FromStr for Month {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 7 {
            return Err(invalid("Informe o mês como AAAA-MM."));
        }
        let date = NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d")
            .map_err(|_| invalid("Mês inválido. Use AAAA-MM."))?;
        Self::from_date(date).validate()
    }
}
pub fn parse_date(text: &str) -> Result<NaiveDate> {
    let date = NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .map_err(|_| invalid("Data inválida. Use AAAA-MM-DD."))?;
    Month::from_date(date).validate()?;
    Ok(date)
}
