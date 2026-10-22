#[test]
fn an_index_of_another_normal_form_reads_and_keeps_its_version() {
    let mut report = builtin_census().index(&[]).expect("index");
    report.normal_form = 7;
    let text = report.to_turtle().expect("renders old version");
    assert!(text.contains("/nf/7/"));
    let read = Index::from_turtle(&text).expect("reader preserves version");
    assert_eq!(read, report);
    assert!(raf_census::check_normal_form(read.normal_form).is_err());
}

#[test]
fn awkward_names_round_trip() {
    let mut report = builtin_census().index(&[]).expect("index");
    report.groups.clear();
    report.units.truncate(1);
    let unit = &mut report.units[0];
    unit.symbol = "demo::r#type@x\"q\\é".into();
    unit.file = "a b/#%?:@\\é.rs".into();
    let text = report.to_turtle().expect("awkward terms");
    assert!(text.contains("%C3%A9") && text.contains("\\\"") && text.contains("\\\\"));
    assert_eq!(
        Index::from_turtle(&text).expect("reads escaped terms"),
        report
    );
    assert_eq!(text, include_str!("fixtures/reports/awkward.ttl"));
}

#[test]
fn two_cfg_alternatives_on_one_line_have_distinct_unit_nodes() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='cfgs'\nversion='0.1.0'\nedition='2024'\n",
        ),
        (
            "src/lib.rs",
            "#[cfg(a)] pub fn f() {} #[cfg(b)] pub fn f() {}",
        ),
    ]);
    let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("cfg alternatives");
    let report = census.index(&[]).expect("index");
    assert_eq!(report.units.len(), 2);
    assert_eq!(report.units[0].symbol, report.units[1].symbol);
    assert_eq!(report.units[0].line, report.units[1].line);
    assert_ne!(report.units[0].start_byte, report.units[1].start_byte);
    assert_eq!(
        Index::from_turtle(&report.to_turtle().expect("renders")).expect("reads"),
        report
    );
}

#[test]
fn the_writer_refuses_a_value_outside_the_shape() {
    let mut baseline = builtin_census().baseline();
    baseline.body_groups[0].members.truncate(1);
    assert!(matches!(
        baseline.to_turtle(),
        Err(Error::Report(ReportError::Missing(_)))
    ));
}

const JOB_SOURCE: &str = "\
pub fn home(a: u32) -> u32 { a + a + a + a + a + a + a + a + a + a + a }
pub fn entry(b: u32) -> u32 { b + b + b + b + b + b + b + b + b + b + b }
pub fn variant(c: u32) -> u32 { c + c + c + c + c + c + c + c + c + c + c }
pub fn copy(d: u32) -> u32 { d + d + d + d + d + d + d + d + d + d + d }
pub const BIG: u32 = 65536;
pub const OTHER: u32 = 7;
";

#[test]
fn the_index_renders_the_classified_matches_of_every_job() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", JOB_SOURCE),
    ]
    .into_iter()
    .collect();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("loads");
    let fingerprint = census.groups()[0].fingerprint.clone();
    let job = Job::new("sum", "demo::home")
        .entry_points(["demo::entry"])
        .variants(["demo::variant"])
        .forbidding(JobForbidden {
            fingerprints: vec![fingerprint],
            constants: vec![0x1_0000],
        });
    let idle = Job::new("idle", "demo::home");
    let index = census
        .index(&census.match_jobs(&[job, idle]))
        .expect("unique jobs");
    assert_eq!(index.jobs.len(), 2);
    let sum = &index.jobs[1];
    let classes: Vec<(&str, MatchClass)> = sum
        .matches
        .iter()
        .map(|m| (m.symbol.as_str(), m.class))
        .collect();
    assert_eq!(
        classes,
        [
            ("demo::BIG", MatchClass::Copy),
            ("demo::copy", MatchClass::Copy),
            ("demo::entry", MatchClass::Home),
            ("demo::home", MatchClass::Home),
            ("demo::variant", MatchClass::Variant),
        ]
    );
    assert_eq!(sum.copies, 2);
    assert!(index.jobs[0].matches.is_empty() && index.jobs[0].copies == 0);
    let text = index.to_turtle().expect("renders");
    assert_eq!(Index::from_turtle(&text).expect("reads"), index);
    assert!(text.contains("census:matchClass census:Variant"));
}

#[test]
fn a_shared_constant_is_an_integer() {
    let manifest = |name: &str| {
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
    };
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n".to_owned(),
        ),
        ("a/Cargo.toml", manifest("a")),
        (
            "a/src/lib.rs",
            "pub const X: u128 = 0x1_0000_0000_0000_0000_0000;\n".to_owned(),
        ),
        ("b/Cargo.toml", manifest("b")),
        (
            "b/src/lib.rs",
            "pub const Y: u128 = 0x1_0000_0000_0000_0000_0000;\n".to_owned(),
        ),
    ]
    .iter()
    .map(|(path, text)| (*path, text.as_str()))
    .collect();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("loads");
    let baseline = census.baseline();
    assert_eq!(baseline.shared_constants.len(), 1);
    let row = &baseline.shared_constants[0];
    assert_eq!(row.value, 0x1_0000_0000_0000_0000_0000);
    assert_eq!(row.packages, 2);
    let text = baseline.to_turtle().expect("renders");
    assert_eq!(Baseline::from_turtle(&text).expect("reads"), baseline);
}

#[test]
fn two_fresh_loads_of_the_fixtures_give_identical_bytes() {
    assert_eq!(
        builtin::builtin().expect("first"),
        builtin::builtin().expect("second")
    );
    let tree = builtin::tree();
    assert_eq!(
        builtin::of(&tree, ".").expect("first"),
        builtin::of(&tree, ".").expect("second")
    );
}

#[test]
fn two_fresh_loads_of_this_repository_give_identical_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let run = || {
        let tree = DiskTree::new(&root).expect("the repository root opens");
        builtin::of(&tree, "crates").expect("the crates load and render")
    };
    let first = run();
    assert!(first.contains("census:normalForm 1"));
    assert_eq!(first, run());
}

#[test]
fn an_absolute_directory_reports_the_same_as_the_relative_spelling() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().expect("the repository root exists");
    let absolute = builtin::of_dir(root.join("crates").to_str().expect("utf-8 path"))
        .expect("the absolute spelling reports");
    let tree = DiskTree::new(&root).expect("the repository root opens");
    assert_eq!(absolute, builtin::of(&tree, "crates").expect("relative"));
}
