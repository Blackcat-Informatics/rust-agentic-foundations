// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Foreign dialect import through the real native JSON/RDF and source paths.

use raf_census::{
    Census, CensusOptions, Ledger, MAX_LEDGER_BYTES, Migration, MigrationInput, migrate_legacy,
};
use raf_source::MemoryTree;

const BASELINE: &str = r#"{"min_tokens":1,"units":2,"isomorphic_groups":[{"fingerprint":"body:0000000000000001","tokens":7,"sanctioned":false,"members":[{"symbol":"demo::home","package":"demo","file":"src/lib.rs","line":2,"shipping":true},{"symbol":"demo::copy","package":"demo","file":"src/lib.rs","line":3,"shipping":true}]}],"shim_groups":[],"hex_tables":[],"shared_constants":[]}"#;

const ROW: &str = "[census]\nmin_tokens=1\n[[job]]\nid='arithmetic'\nsummary='The shared operation'\nhome='demo::home'\nentry_points=[]\nenforced=true\n";

fn tree() -> MemoryTree {
    MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n",
        ),
        (
            "src/lib.rs",
            "/// The operation.\npub fn home(a:u32)->u32 {a+a+a+a}\npub fn copy(b:u32)->u32 {b+b+b+b}",
        ),
    ])
}

fn import(ledger: &str, baseline: &str) -> Result<Migration, raf_census::Error> {
    migrate_legacy(
        &tree(),
        MigrationInput {
            path: "foreign-ledger.toml",
            text: ledger,
        },
        MigrationInput {
            path: "foreign-baseline.json",
            text: baseline,
        },
    )
}

#[test]
fn real_shaped_dialects_share_one_policy_and_one_owned_metadata_extension() {
    for (metadata, forbidden, expected) in [
        (
            "spec='RFC example'\nvectors=['b','a','b']\nbench=['benches/helpers.rs']\nreplaces_external=[]\n",
            "[job.forbidden]\nfingerprints=['body:0000000000000001']\nconstants=['0x7']\nnames=[]\n",
            "spec",
        ),
        (
            "disposition='consolidate'\nowner='runtime'\n",
            "forbidden_fingerprints=['body:0000000000000001']\n",
            "owner",
        ),
        (
            "external=true\n",
            "[job.forbidden]\nfingerprints=['body:0000000000000001']\n",
            "external",
        ),
    ] {
        let result = import(&format!("{ROW}{metadata}{forbidden}"), BASELINE).unwrap();
        assert_eq!(result.unmapped, []);
        let policy = Ledger::from_turtle(&result.ledger).unwrap();
        let census = Census::load(
            &tree(),
            &["."],
            &CensusOptions::default().min_tokens(1).unwrap(),
        )
        .unwrap();
        let home = census
            .unit("demo::home")
            .unwrap()
            .print
            .as_ref()
            .unwrap()
            .body
            .as_ref()
            .unwrap();
        assert_eq!(
            policy.jobs()[0].job.forbidden.fingerprints,
            std::slice::from_ref(home)
        );
        assert_eq!(policy.jobs()[0].job.home, "demo::home");
        assert!(policy.jobs()[0].enforced);
        let extension = policy
            .graph()
            .get(policy.jobs()[0].extension.as_ref().unwrap())
            .unwrap();
        assert!(
            extension
                .values(&format!("urn:raf:census:extension:{expected}"))
                .next()
                .is_some()
        );
        if expected == "spec" {
            assert_eq!(policy.jobs()[0].job.forbidden.constants, [7]);
            let (text, datatype) = extension
                .values("urn:raf:census:extension:vectors")
                .next()
                .unwrap()
                .literal()
                .unwrap();
            assert_eq!(text, r#"["b","a","b"]"#);
            assert_eq!(datatype, format!("{}JSON", raf_rdf::vocab::RDF));
        }
    }
}

#[test]
fn variants_and_counted_distinct_rows_keep_their_declared_contract() {
    let ledger = format!(
        "{ROW}[job.forbidden]\nfingerprints=['body:0000000000000001']\n[[job.variant]]\nsymbol='demo::copy'\nfile='src/lib.rs'\ndetector='isomorphic'\ncriterion='layering'\nanchor='demo::home'\nreason='Separate target implementation'\n[[distinct]]\nmembers=['demo::home','demo::copy']\nkind='body'\ncriterion='layering'\nanchor='demo::home'\nreason='Separate targets'\n"
    );
    let result = import(&ledger, BASELINE).unwrap();
    let policy = Ledger::from_turtle(&result.ledger).unwrap();
    assert_eq!(result.unmapped, []);
    assert_eq!(
        policy.jobs()[0].variants[0].anchor,
        "urn:raf:code:demo::home"
    );
    assert_eq!(policy.jobs()[0].variants[0].file, "src/lib.rs");
    assert_eq!(policy.distinct()[0].group.members()["demo::copy"], 1);
    let repeated = ledger.replace(
        "members=['demo::home','demo::copy']",
        "members=['demo::home','demo::home']",
    );
    let result = import(&repeated, BASELINE).unwrap();
    assert_eq!(
        Ledger::from_turtle(&result.ledger).unwrap().distinct()[0]
            .group
            .members()["demo::home"],
        2
    );
}

#[test]
fn unknown_fields_unmapped_fingerprints_and_unprojectable_metadata_remain_visible() {
    let patterns = import(
        &format!("{ROW}[job.forbidden]\npatterns=['foreign-pattern']\n"),
        BASELINE,
    )
    .unwrap();
    assert_eq!(patterns.unmapped.len(), 1);
    assert_eq!(patterns.unmapped[0].original, "foreign-pattern");
    assert!(
        patterns.unmapped[0]
            .reason
            .contains("native pattern schema")
    );
    assert!(patterns.ledger.contains("foreign-pattern"));
    assert!(patterns.ledger.contains("# unmapped:"));
    let ledger = format!(
        "{ROW}mystery='retain me'\nbench=[inf]\nforbidden_fingerprints=['body:0000000000000002']\n"
    );
    let result = import(&ledger, BASELINE).unwrap();
    assert_eq!(result.unmapped.len(), 3);
    assert!(
        result
            .unmapped
            .iter()
            .any(|row| row.original.contains("retain me") && row.reason.contains("unknown field"))
    );
    assert!(
        result
            .unmapped
            .iter()
            .any(|row| row.original.contains("inf") && row.reason.contains("nonfinite"))
    );
    assert!(
        result
            .unmapped
            .iter()
            .any(|row| row.original == "body:0000000000000002"
                && row.reason.contains("baseline has no member"))
    );
    assert!(result.ledger.contains("# unmapped:"));
    assert!(result.ledger.contains("body:0000000000000002"));
    assert!(!result.ledger.contains("null"));
    let missing_summary = ROW.replace("summary='The shared operation'\n", "");
    let result = import(&missing_summary, BASELINE).unwrap();
    assert_eq!(result.unmapped.len(), 1);
    assert!(result.unmapped[0].location.contains("summary"));
    assert!(
        Ledger::from_turtle(result.ledger.split("# unmapped:").next().unwrap())
            .unwrap_err()
            .to_string()
            .contains("summary")
    );
    let multiline = ROW.replace(
        "summary='The shared operation'",
        "summary=\"\"\"line one\nline two\"\"\"",
    );
    let result = import(&multiline, BASELINE).unwrap();
    assert_eq!(result.unmapped.len(), 1);
    assert!(result.unmapped[0].original.contains("line one\nline two"));
}

#[test]
fn ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success() {
    for ledger in [
        format!("{ROW}forbidden_fingerprints=[]\n[job.forbidden]\nfingerprints=[]\n"),
        "something='not a ledger'".into(),
        "[census]\nnormal_form=1\n".into(),
        "[[job]]\nid='broken'\nhome=''\n".into(),
        "[[job".into(),
    ] {
        let error = import(&ledger, BASELINE).unwrap_err().to_string();
        assert!(error.contains("foreign-ledger.toml"), "{error}");
    }
    for baseline in [
        "VERSION \"1.2\"",
        &BASELINE.replace("\"units\":2", "\"units\":2,\"units\":3"),
    ] {
        let error = import(ROW, baseline).unwrap_err().to_string();
        assert!(error.contains("foreign-baseline.json"), "{error}");
    }
    assert!(import(ROW, BASELINE).is_ok());
}

#[test]
fn both_foreign_inputs_enforce_size_and_exact_container_depth_bounds() {
    let wide = (0..64)
        .map(|index| format!("field_{index}='retained'"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        import(&format!("{wide}\n{ROW}"), BASELINE)
            .unwrap()
            .unmapped
            .len(),
        64
    );
    let oversized = " ".repeat(MAX_LEDGER_BYTES + 1);
    for (ledger, baseline) in [(oversized.as_str(), BASELINE), (ROW, oversized.as_str())] {
        let error = import(ledger, baseline).unwrap_err().to_string();
        assert!(error.contains("exceeds"), "{error}");
    }
    for (count, valid) in [(31usize, true), (32, false)] {
        let nested = format!("{}0{}", "[".repeat(count), "]".repeat(count));
        let ledger = format!("nested={nested}\n{ROW}");
        assert_eq!(import(&ledger, BASELINE).is_ok(), valid);
        let baseline = BASELINE.replace("\"hex_tables\":[]", &format!("\"hex_tables\":{nested}"));
        assert_eq!(import(ROW, &baseline).is_ok(), valid);
    }
}
