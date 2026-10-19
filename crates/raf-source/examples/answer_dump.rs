// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Write every answer the index gives about the programs of the differential
//! test against `rustc`, so that the answers of two trees can be compared
//! with `diff`.
//!
//! ```text
//! cargo run --release -p raf-source --example answer_dump -- OUT \
//!     [--programs N] [--orders K] [--digest] [--jobs J]
//! ```
//!
//! The programs are those of `tests/rustc_oracle.rs`: `N` programs (64
//! unless given) from each of the two seed families, the bases `0xa11ce00`
//! and `0xc0ffee00` with the seeds `base + 1` to `base + 8` taken in turn,
//! and then the committed programs under `fuzz/corpus/rustc-oracle`, in the
//! order of their file names. Each program is loaded as written and in `K`
//! more orders of its declarations (none unless given), drawn for each
//! program afresh from the order seed of the differential test,
//! `0x0de20001`. For each load the file `OUT` holds a line `== <label>`
//! and the answers of `tests/support/answers.rs`: each probe in the three
//! name spaces, `self::<name>` from every module with a `use`, and the
//! exported surface; a refused load is answered by its error. With
//! `--digest` it holds one line for each load instead, the label and a
//! 64-bit FNV-1a digest of the answers. The labels name the seed, the
//! program and the order, or the corpus file and the order, and no path of
//! the machine, so two trees give the same file exactly when they answer
//! alike.
//!
//! The loads run on `J` threads (as many as the machine offers, at most
//! 16, unless given) and are written in the order above. The last line on
//! standard output counts the loads, the refused loads, the answers, those
//! that are an ambiguity and the refused surfaces. No clock is read.

#[path = "../tests/support/answers.rs"]
mod answers;
#[path = "../tests/support/program.rs"]
#[allow(dead_code, reason = "the dump uses part of the generator")]
mod program;
#[path = "../tests/support/random.rs"]
#[allow(dead_code, reason = "the dump uses part of the generator")]
mod random;

use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, thread};

use answers::Counted;
use program::{Program, random_program};
use raf_source::{MemoryTree, Namespace, TargetKind, Workspace};
use random::Draw;

/// The bases of the two seed families.
const BASES: [u64; 2] = [0x0a11_ce00, 0xc0ff_ee00];

/// The seeds of a family: `base + 1` to `base + 8`.
const SEEDS_PER_BASE: u64 = 8;

/// The seed the orders of each program are drawn from.
const ORDER_SEED: u64 = 0x0de2_0001;

/// What the command line asks for.
struct Request {
    out: PathBuf,
    programs: usize,
    orders: usize,
    digest: bool,
    jobs: usize,
}

fn usage() -> String {
    "usage: answer_dump OUT [--programs N] [--orders K] [--digest] [--jobs J]".to_owned()
}

/// Read the command line.
fn request(args: &[String]) -> Result<Request, String> {
    let mut out = None;
    let mut programs = 64;
    let mut orders = 0;
    let mut digest = false;
    let mut jobs = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(16);
    let mut rest = args.iter();
    let number = |flag: &str, value: Option<&String>| -> Result<usize, String> {
        let text = value.ok_or_else(|| format!("{flag} needs a value; {}", usage()))?;
        text.parse()
            .map_err(|error| format!("{flag} {text:?} is not a whole number: {error}"))
    };
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--programs" => programs = number("--programs", rest.next())?,
            "--orders" => orders = number("--orders", rest.next())?,
            "--jobs" => jobs = number("--jobs", rest.next())?,
            "--digest" => digest = true,
            flag if flag.starts_with("--") => {
                return Err(format!("unknown flag {flag}; {}", usage()));
            }
            path if out.is_none() => out = Some(PathBuf::from(path)),
            extra => return Err(format!("unexpected argument {extra:?}; {}", usage())),
        }
    }
    if jobs == 0 {
        return Err("--jobs must be at least 1".to_owned());
    }
    let out = out.ok_or_else(usage)?;
    Ok(Request {
        out,
        programs,
        orders,
        digest,
        jobs,
    })
}

/// The loads answered at once.
const CHUNK: usize = 4096;

/// Where the answers go: the loads waiting to be answered, the file and
/// what the answers held so far.
struct Sink {
    request: Request,
    waiting: Vec<(String, Program)>,
    file: BufWriter<fs::File>,
    total: Counted,
}

impl Sink {
    /// Queue `program` as written and in the orders asked for, answering
    /// the queue when it is full.
    fn push(&mut self, label: &str, program: &Program) -> Result<(), String> {
        self.waiting.push((label.to_owned(), program.clone()));
        let mut draw = Draw::seeded(ORDER_SEED);
        for order in 0..self.request.orders {
            self.waiting.push((
                format!("{label} order {order}"),
                program.shuffled(&mut draw),
            ));
        }
        if self.waiting.len() >= CHUNK {
            self.flush()?;
        }
        Ok(())
    }

    /// Answer the queued loads and write their answers in order.
    fn flush(&mut self) -> Result<(), String> {
        let waiting = std::mem::take(&mut self.waiting);
        let answered = answer_all(&waiting, self.request.jobs, self.request.digest);
        for ((label, _), (text, counted)) in waiting.iter().zip(answered) {
            self.total = self.total.add(counted);
            let written = if self.request.digest {
                writeln!(self.file, "{label} {text}")
            } else {
                write!(self.file, "== {label}\n{text}")
            };
            written
                .map_err(|error| format!("cannot write {}: {error}", self.request.out.display()))?;
        }
        Ok(())
    }
}

/// Answer the programs of both seed families and of the corpus into `sink`.
fn dump(sink: &mut Sink) -> Result<(), String> {
    for base in BASES {
        let seeds: Vec<u64> = (1..=SEEDS_PER_BASE).map(|n| base + n).collect();
        let mut draws: Vec<Draw> = seeds.iter().map(|seed| Draw::seeded(*seed)).collect();
        for n in 0..sink.request.programs {
            let which = n % seeds.len();
            let program = random_program(&mut draws[which]);
            let label = format!("seed {:#x} program {}", seeds[which], n / seeds.len());
            sink.push(&label, &program)?;
        }
    }
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/rustc-oracle");
    let listed = |error: std::io::Error| format!("cannot list {}: {error}", corpus.display());
    let mut names: Vec<String> = fs::read_dir(&corpus)
        .map_err(listed)?
        .map(|entry| {
            entry
                .map_err(listed)?
                .file_name()
                .into_string()
                .map_err(|name| format!("a corpus file name is not UTF-8: {}", name.display()))
        })
        .collect::<Result<_, _>>()?;
    names.sort();
    for name in names {
        let path = corpus.join(&name);
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let program = Program::parse(&text).map_err(|error| format!("{name}: {error}"))?;
        sink.push(&format!("corpus {name}"), &program)?;
    }
    sink.flush()
}

/// The 64-bit FNV-1a digest of `text`.
fn digest(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The answers of every program on `jobs` threads, in order: the text of
/// each, or its digest.
fn answer_all(
    programs: &[(String, Program)],
    jobs: usize,
    digested: bool,
) -> Vec<(String, Counted)> {
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<(String, Counted)>>> =
        Mutex::new(programs.iter().map(|_| None).collect());
    thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                loop {
                    let at = next.fetch_add(1, Ordering::Relaxed);
                    let Some((_, program)) = programs.get(at) else {
                        break;
                    };
                    let (mut text, counted) = answers::answers(program);
                    if digested {
                        text = format!("{:016x}", digest(&text));
                    }
                    results.lock().expect("the results are not poisoned")[at] =
                        Some((text, counted));
                }
            });
        }
    });
    results
        .into_inner()
        .expect("the results are not poisoned")
        .into_iter()
        .map(|answered| answered.expect("every program was answered"))
        .collect()
}

fn run(args: &[String]) -> Result<String, String> {
    let request = request(args)?;
    let file = fs::File::create(&request.out)
        .map_err(|error| format!("cannot create {}: {error}", request.out.display()))?;
    let mut sink = Sink {
        request,
        waiting: Vec::new(),
        file: BufWriter::new(file),
        total: Counted::default(),
    };
    dump(&mut sink)?;
    sink.file
        .flush()
        .map_err(|error| format!("cannot write {}: {error}", sink.request.out.display()))?;
    let total = sink.total;
    Ok(format!(
        "loads={} refused-loads={} answers={} ambiguous={} refused-surfaces={}",
        total.loads, total.refused_loads, total.answers, total.ambiguous, total.refused_surfaces
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("answer_dump: {error}");
            ExitCode::FAILURE
        }
    }
}
