// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use std::{error::Error, io::Write as _, process::ExitCode};
use raf_census::Gate;
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::{CfgAtom, CommittedGitTree, DeclaredCfg, DeclaredConfiguration,
    GitPackageRoots, LoadOptions, LockedCatalog, LockedEdge, LockedManifest,
    RelPath, TargetKind, DiskTree};

const PIN: &str = "5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43";

fn put(graph: &mut Builder, subject: &str, key: &str, object: Term) -> Result<(), Box<dyn Error>> {
    graph.insert(Triple { subject: Term::iri(subject), predicate: Term::iri(if key == "type" {
        vocab::TYPE.into()
    } else {format!("{}{key}", vocab::REPORT)}), object })?;
    Ok(())
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let repository = std::fs::read_to_string("native-repository.txt")?;
    let repository = repository.trim();
    let cfg_file = "compiler-cfg.txt";
    let home = "demo::run".to_owned();
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
    put(&mut graph,"","root",Term::simple_literal("consumer"))?;
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
    if std::env::args().nth(1).as_deref() == Some("--emit-ledger") {
        print!("{}", graph.finish()?.to_text(Format::Turtle)?);
        return Ok(ExitCode::SUCCESS);
    }
    let first = DiskTree::new(".")?;
    let options = LoadOptions::default();
    let catalog = LockedCatalog::load(&first,&RelPath::new("Cargo.lock")?,&options)?;
    let own = catalog.packages().find(|row| row.name()=="demo" && row.source().is_none()).ok_or("consumer lock row")?.id();
    let hash = catalog.packages().find(|row| row.name()=="purrdf-hash" && row.source().is_some_and(|source| source.revision()==Some(PIN))).ok_or("exact provider lock row")?.id();
    let tree = CommittedGitTree::open(&catalog,hash,&repository,&options)?;
    let edges = [LockedEdge::new("dependencies.native",None)];
    let local = [LockedManifest::new(own,RelPath::new("consumer/Cargo.toml")?,&edges).with_configuration(
        DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"demo"))];
    let native = [LockedManifest::new(hash,RelPath::new("crates/hash/Cargo.toml")?,&[]).with_configuration(
        DeclaredConfiguration::new(declared,&[],TargetKind::Lib,"purrdf_hash"))];
    let result = Gate::workspace(".").check_locked(&first,&catalog,&local,&[GitPackageRoots::new(&tree,&native)]);
    // Frozen ordinary BLAKE3 outputs from the official test vectors, whose
    // inputs repeat 0 through 250. Length one is the single byte zero.
    for (input, frozen) in [
        (&b""[..], "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"),
        (&[0][..], "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213"),
    ] {
        let actual = demo::run(input);
        let expected = native::hex::decode(frozen)?;
        assert_eq!(actual.as_bytes().as_slice(), expected);

    }
    let response = raf_census::RegisteredCheck::from_check(&result)?;
    std::io::stdout().lock().write_all(response.to_ntriples()?.as_bytes())?;
    Ok(ExitCode::from(response.exit_code()))
}
