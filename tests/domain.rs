use economic_management::domain::*;

#[test]
fn parses_exact_brazilian_cents() {
    assert_eq!(parse_money("1234,56").unwrap(), 123456);
    assert_eq!(parse_money("-10,05").unwrap(), -1005);
    assert_eq!(parse_money("0").unwrap(), 0);
    assert_eq!(parse_money(" 12,5 ").unwrap(), 1250);
    assert_eq!(money(-123456), "-R$ 1.234,56");
}
#[test]
fn rejects_ambiguous_or_unbounded_money() {
    for input in ["", "NaN", "1e6", "1,234", "1.234,56", "++1", "1,", "99999999999999999999999"] {
        assert!(parse_money(input).is_err(), "{input}");
    }
}
#[test]
fn installments_never_lose_a_cent() {
    assert_eq!((0..3).map(|i| installment(10000,3,i).unwrap()).collect::<Vec<_>>(), vec![3334,3333,3333]);
    for count in 1..=360 { assert_eq!((0..count).map(|i| installment(10001,count,i).unwrap()).sum::<i64>(),10001); }
    for (total,count,index) in [(1,2,0),(10,0,0),(10,1,1),(-10,1,0),(1000,361,0)] { assert!(installment(total,count,index).is_err()); }
}
#[test]
fn calendar_handles_year_boundary_and_rejects_invalid_months() {
    let m:Month="2026-12".parse().unwrap(); assert_eq!(m.next().to_string(),"2027-01");
    assert_eq!(parse_date("2028-02-29").unwrap().to_string(),"2028-02-29");
    assert!(parse_date("2027-02-29").is_err()); assert!("2026-13".parse::<Month>().is_err());
}
