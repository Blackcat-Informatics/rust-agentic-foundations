// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit corpus maintenance; ordinary invocation checks temporary writer output.
//!
//! `cargo run -p raf-source --example corpus_generate` checks the actual shape and
//! oracle writers in owned temporary directories. It neither searches new oracle
//! programs nor writes committed corpus files.
//!
//! Explicit maintenance commands:
//!
//! ```text
//! cargo run -p raf-source --example corpus_generate -- shapes --write fuzz/corpus/parse-source
//! cargo run -p raf-source --example corpus_generate -- oracle --write fuzz/corpus/rustc-oracle
//! ```
//!
//! Oracle maintenance preserves `RAF_ORACLE_PROGRAMS` and the existing generated
//! search, classification and minimizer. It can be expensive and writes its destination.

#[path = "../tests/support/mod.rs"]
mod support;

#[allow(
    dead_code,
    reason = "the shared oracle implementation also supplies selectable integration assertions"
)]
#[path = "../tests/support/oracle.rs"]
mod oracle;

#[path = "../tests/support/corpus_writer.rs"]
mod corpus_writer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => {
            corpus_writer::assert_corpus_writer();
            oracle::assert_corpus_writer();
            println!("checked corpus writers in temporary directories");
        }
        [kind, flag, directory] if flag == "--write" => {
            let directory = std::path::Path::new(directory);
            match kind.to_str() {
                Some("shapes") => corpus_writer::generate_corpus(directory),
                Some("oracle") => oracle::generate_corpus(directory),
                _ => return Err("usage: corpus_generate [shapes|oracle --write DIRECTORY]".into()),
            }
        }
        _ => return Err("usage: corpus_generate [shapes|oracle --write DIRECTORY]".into()),
    }
    Ok(())
}
