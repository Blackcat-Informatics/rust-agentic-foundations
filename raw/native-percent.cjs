const fs=require('fs');const p='crates/raf-workplan/src/coverage/lines.rs';let s=fs.readFileSync(p,'utf8');const start=s.indexOf('/// `digits`, a whole number in decimal, times');const end=s.indexOf('/// A package over a ceiling',start);if(start<0||end<0)throw new Error('exact percent region missing');s=s.slice(0,start)+`/// Whether the exact line count or percentage is within its ceiling.
/// Comparing products avoids rounded division, including when no line is counted.
fn within(orphan: usize, counted: usize, limit: &CeilingLimit) -> bool {
    use purrdf_xsd::{bigint::BigInt, exact::{Decimal, Integer}, numeric::parse_decimal};
    let count = |value: usize| Decimal::from_integer(Integer::from_bigint(BigInt::from_u128(value as u128)));
    let number = |value: &Number| parse_decimal(value.text()).expect("Number retains validated decimal text");
    match limit {
        CeilingLimit::Lines(lines) => count(orphan) <= number(lines),
        CeilingLimit::Percent(percent) => {
            let left = count(orphan).try_mul(&Decimal::from(100)).expect("integer product preserves decimal scale");
            let right = number(percent).try_mul(&count(counted)).expect("integer product preserves decimal scale");
            left <= right
        }
    }
}

`+s.slice(end);fs.writeFileSync(p,s);
const t='crates/raf-workplan/src/coverage/lines/tests.rs';let q=fs.readFileSync(t,'utf8').replace('    assert_eq!(times("125", 8), "1000");\n    assert_eq!(times("0", 7), "0");','    assert!(within(1, 8, &percent("12.5")));\n    assert!(!within(2, 8, &percent("12.5")));\n    assert!(within(0, 3, &percent("-0.000")));\n    assert!(!within(1, 3, &percent("-0.000")));');fs.writeFileSync(t,q);
