// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use std::error::Error;
use raf_census::Gate;
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::{CfgAtom, CommittedGitTree, DeclaredCfg, DeclaredConfiguration,
    GitPackageRoots, LoadOptions, LockedCatalog, LockedEdge, LockedManifest, MemoryTree,
    RelPath, TargetKind};

const PIN: &str = "5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43";

fn put(graph: &mut Builder, subject: &str, key: &str, object: Term) -> Result<(), Box<dyn Error>> {
    graph.insert(Triple { subject: Term::iri(subject), predicate: Term::iri(if key == "type" {
        vocab::TYPE.into()
    } else {format!("{}{key}", vocab::REPORT)}), object })?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args().nth(1).as_deref() == Some("--print-detector-ledger") {
        let mut example = Builder::new();
        put(&mut example, "", "type", Term::iri(format!("{}Ledger", vocab::REPORT)))?;
        put(&mut example, "", "normalForm", Term::typed_literal(raf_census::NORMAL_FORM.to_string(), vocab::INTEGER))?;
        put(&mut example, "", "reportForm", Term::typed_literal(raf_census::REPORT_FORM.to_string(), vocab::INTEGER))?;
        put(&mut example, "", "job", Term::iri("#ceil-div"))?;
        put(&mut example, "#ceil-div", "type", Term::iri(format!("{}LedgerJob", vocab::REPORT)))?;
        for (key, value) in [("jobId", "ceil-div"), ("summary", "Ceiling division."), ("homeSymbol", "demo_util::math::ceil_div"), ("forbiddenFingerprint", "rule:div-ceil-by-hand")] {
            put(&mut example, "#ceil-div", key, Term::simple_literal(value))?;
        }
        put(&mut example, "#ceil-div", "enforced", Term::boolean(true))?;
        let text = example.finish()?.to_text(Format::Turtle)?;
        raf_census::Ledger::from_turtle(&text)?;
        print!("{text}");
        return Ok(());
    }
    let guide = include_str!("../../../docs/capabilities/census-detectors.md");
    let tail = guide.split_once("```turtle\n").ok_or("native ledger example is absent")?.1;
    let end = tail.find("\n```").ok_or("native ledger example is unterminated")?;
    let example = &tail[..=end];
    raf_census::Ledger::from_turtle(example)?;
    println!("documented native detector ledger parsed");
    let repository = std::env::args_os().nth(1).ok_or("supply existing PurRDF Git repository")?;
    let cfg_file = std::env::args_os().nth(2).ok_or("supply captured compiler cfg")?;
    let home = std::env::args().nth(3).unwrap_or_else(|| "demo::run".into());
    let cfg_text = std::fs::read_to_string(cfg_file)?;
    if cfg_text.len() > 32768 {return Err("compiler cfg exceeds acquisition bound".into());}
    let values: Vec<_> = cfg_text.lines().map(|line| {
        match syn::parse_str::<syn::Meta>(line)? {
            syn::Meta::Path(path) => Ok((path.get_ident().ok_or("cfg key")?.to_string(), None)),
            syn::Meta::NameValue(pair) => {
                let syn::Expr::Lit(value) = pair.value else {return Err("cfg literal".into());};
                let syn::Lit::Str(value) = value.lit else {return Err("cfg string".into());};
                Ok((pair.path.get_ident().ok_or("cfg key")?.to_string(), Some(value.value())))
            },
            _ => Err("cfg protocol".into()),
        }
    }).collect::<Result<_, Box<dyn Error>>>()?;
    let atoms: Vec<_> = values.iter().map(|(key,value)| match value {
        Some(value) => CfgAtom::Value(key,value), None => CfgAtom::Name(key),
    }).collect();
    let declared = DeclaredCfg::complete("x86_64-unknown-linux-gnu", &atoms);
    let mut graph = Builder::new();
    put(&mut graph,"","type",Term::iri(format!("{}Ledger",vocab::REPORT)))?;
    put(&mut graph,"","normalForm",Term::typed_literal(raf_census::NORMAL_FORM.to_string(),vocab::INTEGER))?;
    put(&mut graph,"","reportForm",Term::typed_literal(raf_census::REPORT_FORM.to_string(),vocab::INTEGER))?;
    put(&mut graph,"","job",Term::iri("#job"))?;
    put(&mut graph,"#job","type",Term::iri(format!("{}LedgerJob",vocab::REPORT)))?;
    for (key,value) in [("jobId","native-hash"),("summary","Native BLAKE3"),("homeSymbol",home.as_str())] {
        put(&mut graph,"#job",key,Term::simple_literal(value))?;
    }
    put(&mut graph,"#job","enforced",Term::boolean(true))?;
    put(&mut graph,"#job","forbiddenConstant",Term::typed_literal("7",vocab::INTEGER))?;
    put(&mut graph,"#job","preferredProvider",Term::iri("#provider"))?;
    put(&mut graph,"#provider","type",Term::iri(format!("{}PreferredProvider",vocab::REPORT)))?;
    for (key,value) in [("package","purrdf.*"),("consumer","demo::run")] {
        put(&mut graph,"#provider",key,Term::simple_literal(value))?;
    }
    for api in ["native::blake3::Hasher::new", "native::blake3::Hasher::update", "native::blake3::Hasher::finalize"] {
        put(&mut graph,"#provider","entryPoint",Term::simple_literal(api))?;
    }
    let ledger = graph.finish()?.to_text(Format::Turtle)?;
    let manifest = format!("[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nnative={{package='purrdf-hash',git='https://github.com/Blackcat-Informatics/purrdf.git',rev='{PIN}'}}");
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/Cargo.lock"))?;
    let first = MemoryTree::from([
        ("Cargo.toml",manifest.as_str()),("src/lib.rs",include_str!("consumer.rs")),
        ("Cargo.lock",lock.as_str()),("helpers-ledger.ttl",ledger.as_str()),
    ]);
    let options = LoadOptions::default();
    let catalog = LockedCatalog::load(&first,&RelPath::new("Cargo.lock")?,&options)?;
    let own = catalog.packages().find(|row| row.name()=="demo" && row.source().is_none()).ok_or("consumer lock row")?.id();
    let hash = catalog.packages().find(|row| row.name()=="purrdf-hash" && row.source().is_some_and(|source| source.revision()==Some(PIN))).ok_or("exact provider lock row")?.id();
    let tree = CommittedGitTree::open(&catalog,hash,&repository,&options)?;
    let edges = [LockedEdge::new("dependencies.native",None)];
    let local = [LockedManifest::new(own,RelPath::new("Cargo.toml")?,&edges).with_configuration(
        DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"demo"))];
    let native = [LockedManifest::new(hash,RelPath::new("crates/hash/Cargo.toml")?,&[]).with_configuration(
        DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"purrdf_hash"))];
    let report = Gate::workspace(".").check_locked(&first,&catalog,&local,&[GitPackageRoots::new(&tree,&native)])?;
    if !report.is_clean() {return Err(format!("{:?}",report.findings()).into());}
    // Frozen ordinary BLAKE3 outputs from the official test vectors, whose
    // inputs repeat 0 through 250. Length one is the single byte zero.
    for (input, frozen) in [
        (&b""[..], "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"),
        (&[0][..], "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213"),
    ] {
        let actual = demo::run(input);
        let expected = native::hex::decode(frozen)?;
        assert_eq!(actual.as_bytes().as_slice(), expected);
        println!("stateful native BLAKE3 returned frozen bytes for input length {}", input.len());
    }
    println!("locked public Hasher provider gate clean at {PIN}");
    Ok(())
}
