// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exercise configured debt publication only inside an owned scratch workspace.

use super::registration;

use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use purrdf_testkit::TempDir;
use raf_census::{
    DEBT_FILE, DebtRecord, DebtWrite, Gate, GateReport, LedgerExtension, NORMAL_FORM, REPORT_FORM,
};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::{
    DeclaredCfg, DeclaredConfiguration, DiskTree, LoadOptions, LockedCatalog, LockedEdge,
    LockedManifest, LockedPackage, RelPath, TargetKind,
};

fn put(graph: &mut Builder, subject: &str, key: &str, object: Term) -> Result<(), raf_rdf::Error> {
    graph.insert(Triple {
        subject: Term::iri(subject),
        predicate: Term::iri(if key == "type" {
            vocab::TYPE.into()
        } else if key.starts_with("urn:") {
            key.into()
        } else {
            format!("{}{key}", vocab::REPORT)
        }),
        object,
    })
}

fn ledger(owner: &str) -> Result<String, raf_rdf::Error> {
    let mut graph = Builder::new();
    let class = |name: &str| Term::iri(format!("{}{name}", vocab::REPORT));
    put(&mut graph, "", "type", class("Ledger"))?;
    put(&mut graph, "", "normalForm", Term::integer(NORMAL_FORM))?;
    put(&mut graph, "", "reportForm", Term::integer(REPORT_FORM))?;
    for (id, home, enforced) in [
        ("native", "demo::run", true),
        ("number", "demo::number", false),
        ("custom", "demo::number", true),
    ] {
        let node = format!("#{id}");
        put(&mut graph, "", "job", Term::iri(&node))?;
        put(&mut graph, &node, "type", class("LedgerJob"))?;
        put(&mut graph, &node, "jobId", Term::simple_literal(id))?;
        put(
            &mut graph,
            &node,
            "summary",
            Term::simple_literal("Use the home"),
        )?;
        put(&mut graph, &node, "homeSymbol", Term::simple_literal(home))?;
        put(&mut graph, &node, "enforced", Term::boolean(enforced))?;
    }
    put(&mut graph, "#native", "forbiddenConstant", Term::integer(7))?;
    put(
        &mut graph,
        "#native",
        "preferredProvider",
        Term::iri("#provider"),
    )?;
    put(&mut graph, "#provider", "type", class("PreferredProvider"))?;
    for (key, value) in [
        ("package", "provider"),
        ("consumer", "demo::run"),
        ("entryPoint", "provider::api::encode"),
    ] {
        put(&mut graph, "#provider", key, Term::simple_literal(value))?;
    }
    put(
        &mut graph,
        "#number",
        "forbiddenConstant",
        Term::integer(11),
    )?;
    put(
        &mut graph,
        "#custom",
        "forbiddenFingerprint",
        Term::simple_literal("rule:no-local-copy"),
    )?;
    put(&mut graph, "#custom", "extension", Term::iri("#owner"))?;
    put(&mut graph, "#owner", "type", class("Extension"))?;
    put(
        &mut graph,
        "#owner",
        "urn:example:owner",
        Term::iri("#owner/value"),
    )?;
    put(
        &mut graph,
        "#owner/value",
        "type",
        Term::iri("urn:example:Owner"),
    )?;
    put(
        &mut graph,
        "#owner/value",
        "urn:example:name",
        Term::simple_literal(owner),
    )?;
    graph.finish()?.to_text(Format::Turtle)
}

struct Fixture(TempDir);

impl Fixture {
    fn new() -> Result<Self, Box<dyn Error>> {
        let fixture = Self(TempDir::new_in(std::env::temp_dir())?);
        let root = fixture.root();
        for directory in ["consumer/src", "provider/src"] {
            fs::create_dir_all(root.join(directory))?;
        }
        for (path, text) in [
            (
                "Cargo.toml",
                "[workspace]\nmembers=['consumer','provider']\n",
            ),
            (
                "consumer/Cargo.toml",
                "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='provider',path='../provider',version='1.0.0'}\n",
            ),
            (
                "provider/Cargo.toml",
                "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n",
            ),
            (
                "provider/src/lib.rs",
                "pub mod api { pub fn encode(value:u32)->u32 { value } }\n",
            ),
            (
                "Cargo.lock",
                "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=['provider']\n[[package]]\nname='provider'\nversion='1.0.0'\n",
            ),
        ] {
            fs::write(root.join(path), text)?;
        }
        fs::write(root.join("helpers-ledger.ttl"), ledger("demo")?)?;
        fixture.source(&["first", "second"], "")?;
        Ok(fixture)
    }

    fn root(&self) -> &Path {
        self.0.path()
    }

    fn source(&self, names: &[&str], extra: &str) -> Result<(), std::io::Error> {
        let mut text = "/// Native delegation.\npub fn run(value:u32)->u32{renamed::api::encode(value)}\n/// The number home.\npub fn number()->u32{11}\n".to_owned();
        for name in names {
            text.push_str("pub fn ");
            text.push_str(name);
            text.push_str("()->u32{11}\n");
        }
        text.push_str(extra);
        fs::write(self.root().join("consumer/src/lib.rs"), text)
    }

    fn snapshot(&self) -> Result<Vec<Vec<u8>>, std::io::Error> {
        [
            "Cargo.toml",
            "Cargo.lock",
            "consumer/Cargo.toml",
            "consumer/src/lib.rs",
            "provider/Cargo.toml",
            "provider/src/lib.rs",
            "helpers-ledger.ttl",
        ]
        .iter()
        .map(|path| fs::read(self.root().join(path)))
        .collect()
    }

    fn with_context<T>(
        &self,
        action: impl FnOnce(
            Gate,
            &DiskTree,
            &LockedCatalog,
            &[LockedManifest<'_>],
        ) -> Result<T, raf_census::Error>,
    ) -> Result<T, Box<dyn Error>> {
        let tree = DiskTree::new(self.root())?;
        let catalog =
            LockedCatalog::load(&tree, &RelPath::new("Cargo.lock")?, &LoadOptions::default())?;
        let row = |name| {
            catalog
                .packages()
                .find(|row| row.name() == name)
                .ok_or("missing fixture lock row")
                .map(LockedPackage::id)
        };
        let configuration = |name| {
            DeclaredConfiguration::new(
                DeclaredCfg::complete("fixture-target", &[]),
                &[],
                TargetKind::Lib,
                name,
            )
        };
        let edges = [LockedEdge::new("dependencies.renamed", None)];
        let manifests = [
            LockedManifest::new(row("demo")?, RelPath::new("consumer/Cargo.toml")?, &edges)
                .with_configuration(configuration("demo")),
            LockedManifest::new(row("provider")?, RelPath::new("provider/Cargo.toml")?, &[])
                .with_configuration(configuration("provider")),
        ];
        Ok(action(
            registration::registered_gate(self.root()),
            &tree,
            &catalog,
            &manifests,
        )?)
    }

    fn write(&self) -> Result<DebtWrite, Box<dyn Error>> {
        let before = self.snapshot()?;
        let temporary = self.root().join(".census-debt.ttl.write");
        let preexisting = temporary.try_exists()?;
        let result = self.with_context(|gate, _, catalog, manifests| {
            gate.write_debt_locked(catalog, manifests, &[])
        });
        assert_eq!(self.snapshot()?, before);
        if !preexisting {
            assert!(!temporary.try_exists()?);
        }
        result
    }

    fn check(&self) -> Result<GateReport, Box<dyn Error>> {
        self.with_context(|gate, tree, catalog, manifests| {
            gate.check_locked(tree, catalog, manifests, &[])
        })
    }

    fn refuses(&self, previous: &[u8], cause: &str) -> Result<(), Box<dyn Error>> {
        let error = self.write().unwrap_err();
        assert!(error.to_string().contains(cause), "{error}");
        assert_eq!(fs::read(self.root().join(DEBT_FILE))?, previous);
        Ok(())
    }
}

struct PublicationObserver {
    root: PathBuf,
    invoked: Arc<AtomicBool>,
}

impl LedgerExtension for PublicationObserver {
    fn validate(
        &self,
        _: &str,
        _: raf_census::LedgerExtensionGraph<'_>,
    ) -> Option<Result<(), String>> {
        let error = Gate::workspace(&self.root).write_debt().unwrap_err();
        assert!(matches!(
            &error,
            raf_census::Error::Io { source, .. }
                if source.kind() == std::io::ErrorKind::WouldBlock
        ));
        assert!(
            error.to_string().contains("debt publication is busy"),
            "{error}"
        );
        self.invoked.store(true, Ordering::SeqCst);
        None
    }
}

fn coordination(fixture: &Fixture, previous: &[u8]) -> Result<(), Box<dyn Error>> {
    let path = fixture.root().join(".census-debt.ttl.lock");
    let held = fs::OpenOptions::new().read(true).write(true).open(&path)?;
    held.try_lock()?;
    fixture.refuses(previous, "debt publication is busy")?;
    drop(held);
    assert!(path.is_file());
    let reopened = fs::OpenOptions::new().read(true).write(true).open(&path)?;
    reopened.try_lock()?;
    drop(reopened);
    fixture.write()?;
    assert_eq!(fs::read(fixture.root().join(DEBT_FILE))?, previous);

    let invoked = Arc::new(AtomicBool::new(false));
    let before = fixture.snapshot()?;
    fixture.with_context(|gate, _, catalog, manifests| {
        gate.extensions([Box::new(PublicationObserver {
            root: fixture.root().to_owned(),
            invoked: Arc::clone(&invoked),
        }) as Box<dyn LedgerExtension>])
            .write_debt_locked(catalog, manifests, &[])
    })?;
    assert!(invoked.load(Ordering::SeqCst));
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(fs::read(fixture.root().join(DEBT_FILE))?, previous);
    assert!(!fixture.root().join(".census-debt.ttl.write").try_exists()?);
    fixture.write()?;
    assert!(path.is_file());
    Ok(())
}

fn unexpected_coordination() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new()?;
    let path = fixture.root().join(".census-debt.ttl.lock");
    fs::create_dir(&path)?;
    let error = fixture.write().unwrap_err();
    assert!(
        error.to_string().contains("regular coordination file"),
        "{error}"
    );
    assert!(!fixture.root().join(DEBT_FILE).try_exists()?);
    fs::remove_dir(&path)?;
    #[cfg(unix)]
    {
        let target = fixture.root().join("lock-target");
        fs::write(&target, "preserve unexpected target")?;
        std::os::unix::fs::symlink(&target, &path)?;
        let error = fixture.write().unwrap_err();
        assert!(
            error.to_string().contains("regular coordination file"),
            "{error}"
        );
        assert_eq!(fs::read(&target)?, b"preserve unexpected target");
        assert!(!fixture.root().join(DEBT_FILE).try_exists()?);
        fs::remove_file(&path)?;
        fs::remove_file(target)?;
    }
    fixture.write()?;
    assert!(path.is_file());
    Ok(())
}

fn ratchet(fixture: &Fixture) -> Result<Vec<u8>, Box<dyn Error>> {
    let ordinary = registration::registered_gate(fixture.root())
        .write_debt()
        .unwrap_err();
    assert!(
        ordinary
            .to_string()
            .contains("preferred provider requires grounded locked-source"),
        "{ordinary}"
    );
    assert!(!fixture.root().join(DEBT_FILE).try_exists()?);
    assert_eq!(fixture.write()?.jobs, 1);
    assert!(fixture.check()?.is_clean());
    let original = fs::read(fixture.root().join(DEBT_FILE))?;
    assert_eq!(
        DebtRecord::from_turtle(std::str::from_utf8(&original)?)?.job_copies()["number"],
        2
    );
    fixture.write()?;
    assert_eq!(fs::read(fixture.root().join(DEBT_FILE))?, original);
    fixture.source(&["first", "second", "third"], "")?;
    fixture.refuses(&original, "growth cannot be recorded")?;
    fixture.source(&["first"], "")?;
    assert!(!fixture.check()?.is_clean());
    fixture.write()?;
    assert!(fixture.check()?.is_clean());
    let reduced = fs::read(fixture.root().join(DEBT_FILE))?;
    assert_eq!(
        DebtRecord::from_turtle(std::str::from_utf8(&reduced)?)?.job_copies()["number"],
        1
    );
    assert_ne!(original, reduced);
    Ok(reduced)
}

fn refusals(fixture: &Fixture, reduced: &[u8]) -> Result<(), Box<dyn Error>> {
    fixture.source(&["first"], "pub fn local_copy()->u32{2}\n")?;
    fixture.refuses(reduced, "rule no-local-copy")?;
    fixture.source(&["first"], "")?;
    fixture.write()?;
    fixture.source(&["first"], "pub fn competitor()->u32{7}\n")?;
    fixture.refuses(reduced, "competitor")?;
    fixture.source(&["first"], "")?;
    fs::write(fixture.root().join("helpers-ledger.ttl"), ledger("other")?)?;
    fixture.refuses(reduced, "owner extension")?;
    fs::write(fixture.root().join("helpers-ledger.ttl"), ledger("demo")?)?;
    fs::write(fixture.root().join(DEBT_FILE), "malformed prior")?;
    assert!(fixture.write().is_err());
    assert_eq!(
        fs::read(fixture.root().join(DEBT_FILE))?,
        b"malformed prior"
    );
    let foreign = DebtRecord::new(NORMAL_FORM + 1, [], BTreeMap::new())?.to_turtle()?;
    fs::write(fixture.root().join(DEBT_FILE), &foreign)?;
    let temporary = fixture.root().join(".census-debt.ttl.write");
    fs::write(&temporary, "preexisting")?;
    fixture.refuses(foreign.as_bytes(), ".census-debt.ttl.write")?;
    assert_eq!(fs::read(&temporary)?, b"preexisting");
    fs::remove_file(&temporary)?;
    fixture.write()?;
    assert!(fixture.check()?.is_clean());
    fs::write(&temporary, "unchanged")?;
    fixture.write()?;
    assert_eq!(fs::read(&temporary)?, b"unchanged");
    fs::remove_file(temporary)?;
    Ok(())
}

pub(crate) fn exercise() -> Result<(), Box<dyn Error>> {
    unexpected_coordination()?;
    let fixture = Fixture::new()?;
    let reduced = ratchet(&fixture)?;
    coordination(&fixture, &reduced)?;
    refusals(&fixture, &reduced)?;
    println!("locked adoption, exact bytes, ratchet refusals and serialized publication pass");
    Ok(())
}
