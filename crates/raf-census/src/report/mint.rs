// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Injective fragment names, using the shared RFC 3986 percent encoder.
use purrdf_lex::percent;

fn component(bytes: &[u8]) -> Option<usize> {
    bytes
        .iter()
        .position(|byte| !byte.is_ascii_alphanumeric() && !b"-._~!$&'()*+,;=:".contains(byte))
}
pub(super) fn part(text: &str) -> String {
    percent::encode(text, component).into_owned()
}
pub(super) fn unit(symbol: &str, file: &str, line: usize, start: usize) -> String {
    format!(
        "#unit/{}@{}:{line}:{start}",
        part(symbol),
        percent::encode(file, percent::PATH)
    )
}
pub(super) fn job(id: &str) -> String {
    format!("#job/{}", part(id))
}
pub(super) fn reifier(id: &str, unit: &str) -> String {
    format!(
        "#match/{}/{}",
        part(id),
        unit.strip_prefix("#unit/").expect("unit fragment")
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fragments_are_injective_over_their_separators() {
        assert_ne!(unit("a@b", "c", 1, 0), unit("a", "b@c", 1, 0));
        assert_ne!(unit("a", "c", 1, 0), unit("a", "c", 1, 2));
        assert_ne!(job("a/b"), job("a%2Fb"));
    }
    #[test]
    fn fragments_are_ascii_and_percent_encode_reserved_bytes() {
        assert_eq!(part("é@/#?%"), "%C3%A9%40%2F%23%3F%25");
        assert!(unit("é", "a b/#%", 1, 0).is_ascii());
    }
}
