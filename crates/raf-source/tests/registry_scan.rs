// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Selectable assertions of the actual registry-scanning example.

#[allow(
    dead_code,
    reason = "the example entrypoint and orchestration are not called by every fixture test"
)]
#[path = "../examples/registry_scan/driver.rs"]
mod registry_scan;
#[cfg(test)]
mod tests {
    use super::registry_scan::{
        Tally, Version, child_args, load_one, ok_line, parse_args, parse_bytes, parse_one,
        parse_work,
    };
    use raf_source::{Budget, Usage};
    use std::fmt::Write as _;
    use std::path::Path;

    fn strings(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn work_limit_refuses_zero_and_names_the_flag() {
        let refused = parse_work("0").unwrap_err();
        assert!(refused.contains("--max-work"), "{refused}");
        assert_eq!(parse_work("1"), Ok(1));
        assert_eq!(parse_work("1000"), Ok(1000));
        for bad in ["", "-1", "ten", "1.5"] {
            let refused = parse_work(bad).unwrap_err();
            assert!(refused.contains("--max-work"), "{refused}");
        }
    }

    #[test]
    fn child_arguments_carry_the_directory_and_the_limit() {
        let args = strings(&["--one", "/r/a-1.0.0", "--max-work", "42"]);
        assert_eq!(parse_one(&args), Some(("/r/a-1.0.0", 42)));
        for refused in [
            strings(&["--one", "/r/a-1.0.0"]),
            strings(&["--one", "/r/a-1.0.0", "--max-work", "many"]),
            strings(&["--one", "/r/a-1.0.0", "--jobs", "4"]),
            strings(&["/r", "--max-work", "5"]),
            strings(&[]),
        ] {
            assert_eq!(parse_one(&refused), None, "{refused:?}");
        }
    }

    #[test]
    fn version_components_and_prerelease_precedence_keep_their_width_and_order() {
        let version = |text| Version::parse(text).expect("a version");
        let ordered = [
            "1.2.3-alpha.1",
            "1.2.3-alpha.2",
            "1.2.3-alpha.10",
            "1.2.3-alpha.18446744073709551615",
            "1.2.3-alpha.18446744073709551616",
            "1.2.3-alpha.beta",
            "1.2.3-beta",
            "1.2.3",
            "1.2.4",
        ];
        for pair in ordered.windows(2) {
            assert!(version(pair[0]) < version(pair[1]), "{pair:?}");
        }
        assert_eq!(version("1.2.3+build"), version("1.2.3"));
        assert_eq!(version("01.002.0003"), version("1.2.3"));
        assert_eq!(
            version("1.2.3-alpha.0002").cmp(&version("1.2.3-alpha.2")),
            std::cmp::Ordering::Equal
        );
        assert_eq!(version("18446744073709551615.0.0").core[0], u64::MAX);
        for invalid in [
            "18446744073709551616.0.0",
            "+1.2.3",
            "1.2",
            "1.2.3.4",
            "1.2.-3",
        ] {
            assert_eq!(Version::parse(invalid), None, "{invalid}");
        }
    }

    #[test]
    fn count_callers_preserve_padding_width_suffixes_and_misplaced_totals() {
        let padded = format!("+{}42", "0".repeat(256));
        assert_eq!(parse_work(&padded), Ok(42));
        assert_eq!(parse_work(&usize::MAX.to_string()), Ok(usize::MAX));
        assert_eq!(parse_bytes("+0001M"), Ok(1 << 20));
        assert_eq!(parse_bytes(&u64::MAX.to_string()), Ok(u64::MAX));
        assert!(parse_bytes("18446744073709551616").is_err());
        assert!(parse_work(&(usize::MAX as u128 + 1).to_string()).is_err());
        let options = parse_args(&strings(&["/r", "--jobs", "+0002", "--max-memory", "1M"]))
            .expect("arguments on a host with at least two mebibytes available");
        assert_eq!(options.jobs, 2);
        let mut tally = Tally::default();
        tally.count("a", "OK misplaced=2 work=18446744073709551615");
        tally.count("b", "OK misplaced=3 work=00042");
        assert_eq!(tally.misplaced_items, 5);
        assert_eq!(tally.with_misplaced, 2);
        assert_eq!(tally.largest.get("work"), Some(&(u64::MAX, "a".to_owned())));
    }

    #[test]
    fn the_child_is_given_the_limit_it_was_asked_for() {
        for limit in [1, 42, Budget::Work.limit() - 1] {
            let args: Vec<String> = child_args(Path::new("/r/a-1.0.0"), limit)
                .into_iter()
                .map(|arg| arg.into_string().unwrap())
                .collect();
            assert_eq!(args[2], "--max-work");
            assert_eq!(args[3], limit.to_string());
            assert_eq!(parse_one(&args), Some(("/r/a-1.0.0", limit)));
        }
    }

    #[test]
    fn ok_line_prints_work_last_among_the_usage_figures() {
        let mut usage = Usage::default();
        usage.work = 12;
        usage.max_directory_depth = 3;
        let line = ok_line(5, 2, &usage, &[]);
        assert_eq!(
            line,
            "OK items=5 files=2 misplaced=0 total=0 item=0 fact=0 glob=0 tokens=0 recursion=0 depth=3 work=12"
        );
        let line = ok_line(5, 2, &usage, &["a::b in src/x.rs".to_owned()]);
        assert!(line.ends_with("work=12 a::b in src/x.rs"), "{line}");
    }

    #[test]
    fn tally_keeps_the_largest_work_and_counts_budget_refusals() {
        let mut usage = Usage::default();
        let mut tally = Tally::default();
        usage.work = 0;
        tally.count("tiny@1.0.0", &ok_line(1, 1, &usage, &[]));
        usage.work = 900;
        tally.count("big@2.0.0", &ok_line(1, 1, &usage, &[]));
        usage.work = 30;
        tally.count("mid@1.0.0", &ok_line(1, 1, &usage, &[]));
        tally.count(
            "hub@1.0.0",
            "ERR Budget :: src/lib.rs:3: exceeds the budget for glob table work steps (1000)",
        );
        assert_eq!(tally.ok, 3);
        assert_eq!(tally.errors.get("Budget"), Some(&1));
        assert_eq!(
            tally.largest.get("work"),
            Some(&(900, "big@2.0.0".to_owned()))
        );
    }

    /// A crate directory holding a hub of `n` globs, in a scratch directory
    /// owned by the native fixture library.
    fn hub_crate(n: usize) -> purrdf_testkit::TempDir {
        let owned =
            purrdf_testkit::TempDir::new_in(std::env::temp_dir()).expect("a scratch directory");
        let dir = owned.path();
        std::fs::create_dir_all(dir.join("src")).expect("create the scratch crate");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"hub\"\nversion = \"1.0.0\"\nedition = \"2021\"\n",
        )
        .expect("write the manifest");
        let mut lib = String::from("pub mod hub {");
        for i in 0..n {
            let vis = if i % 2 == 0 { "pub " } else { "" };
            let _ = write!(lib, " {vis}use crate::s{i}::*;");
        }
        lib.push_str(" }\n");
        for i in 0..n {
            let _ = writeln!(
                lib,
                "pub mod s{i} {{ pub use crate::hub::*; pub struct T{i}; }}"
            );
        }
        std::fs::write(dir.join("src/lib.rs"), lib).expect("write the source");
        owned
    }

    #[test]
    fn the_work_limit_reaches_the_load() {
        let dir = hub_crate(40);
        let large = load_one(dir.path(), Budget::Work.limit());
        let small = load_one(dir.path(), 1000);
        assert!(large.starts_with("OK "), "{large}");
        assert!(small.starts_with("ERR Budget :: "), "{small}");
    }
}
