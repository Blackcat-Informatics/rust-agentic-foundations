// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Foreign dialect import through the real native JSON/RDF and source paths.

use raf_census::{
    Census, CensusOptions, ForeignField, ForeignLedgerAdapter, Ledger, MAX_LEDGER_BYTES, Migration,
    MigrationInput, migrate_legacy, migrate_legacy_with_extensions,
};
use raf_source::MemoryTree;
use std::{collections::BTreeSet, sync::Mutex};

const PROOF_FIELDS: &str = "urn:example:capacity-proof";
const ORIGINAL: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

#[derive(Default)]
struct ProofAdapter {
    expected: Mutex<Vec<raf_rdf::Triple>>,
}

impl ForeignLedgerAdapter for ProofAdapter {
    fn claim(&self, _: &str, field: ForeignField<'_, '_>) -> Option<Result<String, String>> {
        if field.node().key.as_ref()?.decoded != "capacity_proof" {
            return None;
        }
        if !matches!(
            field.node().kind,
            purrdf_toml::Kind::Array | purrdf_toml::Kind::ArrayOfTables
        ) || field.node().children.iter().any(|id| {
            !matches!(
                field.document().nodes()[*id].kind,
                purrdf_toml::Kind::Table(_)
            )
        }) {
            return Some(Err("capacity proof must contain rows".into()));
        }
        let result = (|| {
            let native = purrdf_toml::project(field.document(), field.profile())
                .map_err(|error| error.to_string())?;
            let mut selected = BTreeSet::from([field.document().id().to_owned()]);
            let mut pending = vec![field.occurrence()];
            while let Some(id) = pending.pop() {
                selected.insert(format!("{}#v{id}", field.document().id()));
                pending.extend(&field.document().nodes()[id].children);
            }
            *self.expected.lock().unwrap() = native
                .quads()
                .map(|fact| raf_rdf::Triple {
                    subject: native.term_value(fact.s),
                    predicate: native.term_value(fact.p),
                    object: native.term_value(fact.o),
                })
                .filter(|fact| matches!(&fact.subject, raf_rdf::TermValue::Iri(id) if selected.contains(id)))
                .collect();
            Ok(PROOF_FIELDS.into())
        })();
        Some(result)
    }
}

struct ClaimField(&'static str, Result<&'static str, &'static str>);

impl ForeignLedgerAdapter for ClaimField {
    fn claim(&self, _: &str, field: ForeignField<'_, '_>) -> Option<Result<String, String>> {
        (field.node().key.as_ref()?.decoded == self.0)
            .then(|| self.1.map(str::to_owned).map_err(str::to_owned))
    }
}

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
    claimed_policy_values_keep_complete_native_facts();
}

fn claimed_policy_values_keep_complete_native_facts() {
    let proof = "{file='src/lib.rs',source='source:original-proof-pin',line=0x2a,authorities=['A','A'],created=1979-05-27T07:32:00Z}";
    let foreign = format!("{ROW}owner='runtime'\ncapacity_proof=[{proof},{proof}]\n");
    let adapter = ProofAdapter::default();
    let run = |text: &str, adapters: &[&dyn ForeignLedgerAdapter]| {
        migrate_legacy_with_extensions(
            &tree(),
            MigrationInput {
                path: "foreign-ledger.toml",
                text,
            },
            MigrationInput {
                path: "foreign-baseline.json",
                text: BASELINE,
            },
            adapters,
        )
    };
    let result = run(&foreign, &[&adapter]).unwrap();
    assert_eq!(result.unmapped, []);
    let policy = Ledger::from_turtle(&result.ledger).unwrap();
    let extension = policy
        .extension(policy.jobs()[0].extension.as_deref().unwrap())
        .unwrap();
    assert_eq!(
        extension
            .root()
            .values("urn:raf:census:extension:owner")
            .next()
            .unwrap()
            .literal()
            .unwrap()
            .0,
        "runtime"
    );
    let root = extension
        .root()
        .values(PROOF_FIELDS)
        .next()
        .unwrap()
        .iri()
        .unwrap();
    assert_eq!(
        extension
            .get(root)
            .unwrap()
            .values(purrdf_core::vocab::rdfs::MEMBER)
            .count(),
        2
    );

    assert_retained_native_facts(&policy, extension, &adapter.expected.lock().unwrap());
    assert!(result.ledger.contains("source:original-proof-pin"));
    assert_ne!(import(&foreign, BASELINE).unwrap().unmapped, []);
    let unclaimed = run(&format!("{foreign}mystery='retain me'\n"), &[&adapter]).unwrap();
    assert_eq!(unclaimed.unmapped.len(), 1);
    assert!(unclaimed.unmapped[0].location.contains("mystery"));
    assert!(unclaimed.unmapped[0].original.contains("retain me"));

    let core = ClaimField("id", Ok(PROOF_FIELDS));
    let relative = ClaimField("capacity_proof", Ok("#relative"));
    let reserved = ClaimField("capacity_proof", Ok(raf_rdf::vocab::TYPE));
    let report = ClaimField("capacity_proof", Ok(raf_rdf::vocab::REPORT));
    let refusal = ClaimField("capacity_proof", Err("source schema refused the proof"));
    for (claims, location, reason) in [
        (
            vec![&adapter as &dyn ForeignLedgerAdapter, &refusal],
            "capacity_proof",
            "multiple",
        ),
        (vec![&core], "id", "core-owned"),
        (vec![&relative], "capacity_proof", "IRI"),
        (vec![&reserved], "capacity_proof", "rdf:type"),
        (vec![&report], "capacity_proof", "core report vocabulary"),
        (vec![&refusal], "capacity_proof", "source schema refused"),
    ] {
        let error = run(&foreign, &claims).unwrap_err().to_string();
        assert!(error.contains(location), "{error}");
        assert!(error.contains(reason), "{error}");
    }
    let malformed = run(&format!("{ROW}capacity_proof=[7]\n"), &[&adapter])
        .unwrap_err()
        .to_string();
    assert!(malformed.contains("capacity_proof") && malformed.contains("must contain rows"));
}

fn assert_retained_native_facts(
    policy: &Ledger,
    extension: raf_census::LedgerExtensionGraph<'_>,
    expected: &[raf_rdf::Triple],
) {
    let inverse: std::collections::BTreeMap<_, _> = extension
        .nodes()
        .filter_map(|(local, node)| {
            node.values(ORIGINAL)
                .next()
                .and_then(raf_rdf::Value::iri)
                .map(|original| (local.to_owned(), original.to_owned()))
        })
        .collect();
    let restore = |mut term: raf_rdf::TermValue| {
        if let raf_rdf::TermValue::Iri(iri) = &mut term {
            let local = policy.graph().local(iri);
            if let Some(original) = inverse.get(local) {
                original.clone_into(iri);
            }
        }
        term
    };
    let actual: Vec<_> = policy.graph().assertions().filter(|fact| {
        matches!(&fact.subject, raf_rdf::TermValue::Iri(id) if inverse.contains_key(policy.graph().local(id)))
            && !matches!(&fact.predicate, raf_rdf::TermValue::Iri(id) if id == ORIGINAL || id == purrdf_core::vocab::rdfs::MEMBER)
    }).map(|fact| raf_rdf::Triple {
        subject: restore(fact.subject), predicate: fact.predicate, object: restore(fact.object),
    }).collect();
    assert_eq!(actual.len(), expected.len());
    assert!(
        expected.iter().all(|fact| actual.contains(fact)),
        "every original native fact must survive relocation"
    );
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
    nested_metadata_remains_visible();
}

fn nested_metadata_remains_visible() {
    let nested = import(
        &format!("{ROW}mystery={{nested=['repeat','repeat'],empty={{}}}}\n[unrecognized]\nwhen=1979-05-27T07:32:00-07:00\n"),
        BASELINE,
    ).unwrap();
    let repeats: Vec<_> = nested
        .unmapped
        .iter()
        .filter(|row| row.original == "'repeat'")
        .collect();
    assert_eq!(repeats.len(), 2);
    assert_ne!(repeats[0].location, repeats[1].location);
    for original in ["{}", "[unrecognized]", "1979-05-27T07:32:00-07:00"] {
        assert!(
            nested.unmapped.iter().any(|row| row.original == original),
            "{original}: {:?}",
            nested.unmapped
        );
    }
    let table = import(
        &format!("{ROW}[job.ext.record]\nwhen=1979-05-27T07:32:00-07:00\nproof=['first','second','first']\n"),
        BASELINE,
    ).unwrap();
    assert!(
        table
            .unmapped
            .iter()
            .any(|row| row.original.contains("1979-05-27T07:32:00-07:00")
                && row.original.matches("'first'").count() == 2)
    );
    let aot = format!(
        "{ROW}[[job.ext.proofs]]\nwhen=1979-05-27T07:32:00-07:00\nproof=['first','first']\n[[job.ext.proofs]]\nreason='second'\n"
    );
    let result = import(&aot, BASELINE).unwrap();
    assert!(result.unmapped.iter().any(|row| {
        row.original.matches("[[job.ext.proofs]]").count() == 2
            && row.original.contains("1979-05-27T07:32:00-07:00")
            && row.original.matches("'first'").count() == 2
            && row.original.contains("'second'")
    }));
    let projectable = import(
        &aot.replace("when=1979-05-27T07:32:00-07:00", "when='dated'"),
        BASELINE,
    )
    .unwrap();
    assert_eq!(projectable.unmapped, []);
}

#[test]
fn ambiguous_aliases_bad_layouts_and_wrong_report_inputs_refuse_without_partial_success() {
    let malformed = import("[[job", BASELINE).unwrap_err();
    let mut cause = std::error::Error::source(&malformed);
    let mut native = false;
    while let Some(error) = cause {
        native |= error.is::<purrdf_toml::TomlError>();
        cause = error.source();
    }
    assert!(native, "native TOML cause was lost: {malformed}");
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
    for literal in [
        "9223372036854775808",
        "0x8000000000000000",
        "1e999",
        "-1e999",
    ] {
        assert!(
            import(&format!("ignored={literal}\n{ROW}"), BASELINE).is_err(),
            "{literal}"
        );
    }
    for literal in [
        "9223372036854775807",
        "-9223372036854775808",
        "-0.0",
        "inf",
        "+inf",
        "-inf",
        "nan",
        "+nan",
        "-nan",
    ] {
        assert!(
            import(&format!("ignored={literal}\n{ROW}"), BASELINE).is_ok(),
            "{literal}"
        );
    }
}
