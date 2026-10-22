const fs=require('fs');let p='crates/cargo-workplan/src/main.rs',s=fs.readFileSync(p,'utf8');s=s.replace('text|json','text|turtle|ntriples').replace('in text and in JSON','in text and in RDF');let a=s.indexOf('//! always written to standard error. `--format json`'),b=s.indexOf('//! # Exit codes',a);if(a<0||b<0)throw Error('main section');
s=s.slice(0,a)+`//! always written to standard error. Output and diagnostic write failures are
//! run failures, exit 2, including a note that cannot be written.
//!
//! # Machine reports
//!
//! \x60--format turtle\x60 and \x60--format ntriples\x60 emit the closed canonical RDF 1.2
//! report profile documented in \x60docs/capabilities/rdf-report-profile.md\x60.
//! Both formats encode the same typed graph and snapshot identity. JSON formats
//! are usage errors. A successful run with no visible result still emits a
//! canonical Report root, command and reportForm, with no result members.
//!
//! Every command returns typed domain results. Named rows and zero-based
//! ordinals preserve ordered lists: capability groups, bindings, guidance,
//! references, definitions, measures, inputs, claims, commits, paths and ranges.
//! Optional fields are absent when they do not apply. Findings retain their
//! location variant and fix; histories retain full commits; input observations
//! retain actual digests. Counts and booleans are typed literals; measured
//! decimals remain exact decimal text, without conversion to floating point.
//!
//! Verification includes only the invocation rows returned by the executor,
//! their ordered arguments and process outcomes. A planning refusal has no
//! invocation row. Snapshot-local rows do not claim timestamps or independent
//! execution identifiers. Coverage source-location reifiers describe observed
//! item declarations, without asserting a resolved definition or a test run.
//!
`+s.slice(b);
s=s.replace('the table and the JSON array of `raf_workplan::history::render`','a human table or typed RDF').replace('as the table and the JSON array of\n//!   `raf_workplan::coverage::render`','as a human table or typed RDF').replace('as the table and the JSON array of\n//!   `raf_workplan::coverage::render::claims_table` and `claims_json`','as a human table or typed RDF').replace('as the tables and the JSON array of\n//!   `raf_workplan::coverage::render::lines_table` and `lines_json`','as human tables or typed RDF');fs.writeFileSync(p,s);
for (const name of ['assess','evidence','coverage','history']) { p='crates/cargo-workplan/src/'+name+'.rs';s=fs.readFileSync(p,'utf8').replace('the JSON form escapes as\n/// JSON does.','the RDF form retains the typed text.').replace('the JSON form escapes as JSON does.','the RDF form retains the typed text.').replace('printed as the table and the JSON array of\n//! `raf_workplan::coverage::render`','printed as a human table or a typed RDF graph').replace('printed as the table\n//! and the JSON array of `raf_workplan::history::render`','printed as a human table or a typed RDF graph');fs.writeFileSync(p,s); }
p='crates/cargo-workplan/src/cli.rs';s=fs.readFileSync(p,'utf8').replace('How findings are printed: text, or one JSON array for the run.','Output carrier: human text, canonical Turtle 1.2 or canonical N-Triples 1.2.').replace('a_format_that_is_not_text_or_json_is_a_usage_error','an_unknown_or_json_format_is_a_usage_error');s=s.replace('        let error = parsed(&["cargo-workplan", "--format", "yaml", "check"]).expect_err("yaml");\n        assert_eq!(error.exit_code(), 2);',`        for format in ["yaml", "json", "json-pretty", "JSON"] {
            let error = parsed(&["cargo-workplan", "--format", format, "check"]).expect_err("unsupported format");
            assert_eq!(error.exit_code(), 2);
        }
        for (name, format) in [("text", Format::Text), ("turtle", Format::Turtle), ("ntriples", Format::Ntriples)] {
            assert_eq!(same(&["--format", name, "check"]).format, format);
        }`);fs.writeFileSync(p,s);
p='crates/raf-workplan/src/lib.rs';s=fs.readFileSync(p,'utf8').replace('[`coverage::render`] writes each direction as a table and as JSON.','[`coverage::render`] writes each direction as a human table; [`report`] projects typed findings and item coverage as RDF.').replace('[`history::render`] writes the table and the JSON form of the','[`history::render`] writes the human table of the');fs.writeFileSync(p,s);
p='crates/raf-workplan/src/history.rs';s=fs.readFileSync(p,'utf8').replace('[`render`] writes the table and the JSON form of the same entries.','[`render`] writes a human table of the same typed entries.');fs.writeFileSync(p,s);
p='crates/raf-workplan/src/coverage.rs';s=fs.readFileSync(p,'utf8').replace('The name of the failure as the JSON form writes it.','The stable name of the failure.');fs.writeFileSync(p,s);
