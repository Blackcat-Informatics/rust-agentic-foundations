// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The caller's own command: the one process whose argv is not a row of an
//! argv table.

use std::error::Error;
use std::ffi::OsString;
use std::fmt;

/// One word of a [`TrustedArgv`], as an argument of an
/// [`Invocation`](super::Invocation).
///
/// The constructor is private to the crate and is called only by
/// [`TrustedArgv`], so a word holds text the caller typed on the command line
/// and nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustedWord(String);

impl TrustedWord {
    /// The text of the word.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrustedWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The command a person or an agent typed after `--` on the command line of
/// `evidence record`: the program and its arguments, run exactly as given.
///
/// It is trusted build input, as the workspace's own build commands are: it
/// runs with no shell, its standard input null, its output bounded and a
/// wall-clock limit, in the workspace root, with the caller's environment
/// unchanged (no `LC_ALL=C`, and no `--frozen` added to a `cargo` in it). It is
/// the one process of the workplan whose argv is not a row of an argv table,
/// and it can be built from one place only: the command line of the running
/// process ([`TrustedArgv::from_command_line`] takes the [`std::env::ArgsOs`]
/// that only [`std::env::args_os`] makes). The text of a loaded document or
/// record, such as the `command` an evidence record stores for its readers,
/// cannot become one:
///
/// ```compile_fail,E0308
/// use raf_workplan::exec::TrustedArgv;
///
/// // The `command` key of a loaded evidence record is text, and text is not
/// // the command line of the process.
/// let command: Option<String> = Some("cargo bench parse".to_string());
/// let _ = TrustedArgv::from_command_line(command.unwrap_or_default());
/// ```
///
/// ```compile_fail,E0451
/// use raf_workplan::exec::TrustedArgv;
///
/// // Nor can its words be filled in directly.
/// let _ = TrustedArgv { words: Vec::new() };
/// ```
///
/// ```compile_fail,E0624
/// use raf_workplan::exec::TrustedArgv;
///
/// // And the crate's own constructor, for its tests, is not public.
/// let _ = TrustedArgv::new(vec!["sh".to_string()]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustedArgv {
    words: Vec<TrustedWord>,
}

/// The word that ends the options of `evidence record` and starts the command.
const SEPARATOR: &str = "--";

impl TrustedArgv {
    /// The words after the first `--` of the command line of this process.
    ///
    /// # Errors
    ///
    /// [`TrustedArgvError::NoSeparator`] when no word is `--`,
    /// [`TrustedArgvError::Empty`] when nothing follows it,
    /// [`TrustedArgvError::EmptyProgram`] when the program is the empty word,
    /// and [`TrustedArgvError::NotUtf8`] when a word of the command is not
    /// UTF-8 (the record stores the command as text for its readers, so a word
    /// it cannot hold is refused rather than altered).
    pub fn from_command_line(args: std::env::ArgsOs) -> Result<Self, TrustedArgvError> {
        Self::after_separator(args)
    }

    /// The words after the first `--` of `args`.
    fn after_separator(args: impl IntoIterator<Item = OsString>) -> Result<Self, TrustedArgvError> {
        let mut args = args.into_iter();
        if !args.any(|arg| arg == SEPARATOR) {
            return Err(TrustedArgvError::NoSeparator);
        }
        let words = args
            .enumerate()
            .map(|(index, arg)| {
                arg.into_string()
                    .map_err(|_| TrustedArgvError::NotUtf8 { index })
            })
            .collect::<Result<Vec<String>, _>>()?;
        Self::new(words)
    }

    /// The command of these words, as the crate's tests build one.
    pub(crate) fn new(words: Vec<String>) -> Result<Self, TrustedArgvError> {
        match words.first() {
            None => return Err(TrustedArgvError::Empty),
            Some(program) if program.is_empty() => return Err(TrustedArgvError::EmptyProgram),
            Some(_) => {}
        }
        Ok(Self {
            words: words.into_iter().map(TrustedWord).collect(),
        })
    }

    /// The words: the program, then its arguments.
    #[must_use]
    pub fn words(&self) -> &[TrustedWord] {
        &self.words
    }

    /// The program: the first word.
    #[must_use]
    pub fn program(&self) -> &str {
        self.words.first().map_or("", TrustedWord::as_str)
    }

    /// The command as text for a reader: the words joined by one space, each
    /// that holds anything but `A-Z a-z 0-9` and `_ @ % + = : , . / -` (or is
    /// empty) in single quotes, a `'` in it written `'\''`, as a POSIX shell
    /// would read the text back. It is never run: the workplan starts no shell.
    #[must_use]
    pub fn text(&self) -> String {
        text_of(self.words.iter().map(TrustedWord::as_str))
    }
}

/// The words as [`TrustedArgv::text`] writes them, each quoted as a POSIX
/// shell reads it back and joined by one space. The message of an error about
/// the caller's command quotes it with this, so it reads as the record stores
/// it.
pub(crate) fn text_of<'a>(words: impl IntoIterator<Item = &'a str>) -> String {
    words.into_iter().map(quote).collect::<Vec<_>>().join(" ")
}

/// `word` as a POSIX shell reads it back.
fn quote(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    if plain {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

/// Why the command line holds no command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrustedArgvError {
    /// No word of the command line is `--`.
    NoSeparator,
    /// Nothing follows the `--`.
    Empty,
    /// The program is the empty word.
    EmptyProgram,
    /// A word of the command is not UTF-8.
    NotUtf8 {
        /// The place of the word in the command, from 0 for the program.
        index: usize,
    },
}

impl fmt::Display for TrustedArgvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSeparator => f.write_str(
                "no command to run: put the command after `--`, as in `-- cargo bench parse`",
            ),
            Self::Empty => f.write_str("nothing follows `--`: name the command to run after it"),
            Self::EmptyProgram => f.write_str("the program after `--` is the empty word"),
            Self::NotUtf8 { index } => write!(
                f,
                "word {index} of the command is not UTF-8, and the record stores the command as text; nothing was run"
            ),
        }
    }
}

impl Error for TrustedArgvError {}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{TrustedArgv, TrustedArgvError};

    fn os(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_command_is_every_word_after_the_first_separator() {
        let argv = TrustedArgv::after_separator(os(&[
            "cargo-workplan",
            "workplan",
            "evidence",
            "record",
            "--id",
            "x",
            "--",
            "cargo",
            "bench",
            "--",
            "--save",
        ]))
        .expect("a command");
        let words: Vec<&str> = argv
            .words()
            .iter()
            .map(super::TrustedWord::as_str)
            .collect();
        assert_eq!(words, ["cargo", "bench", "--", "--save"]);
        assert_eq!(argv.program(), "cargo");
    }

    #[test]
    fn a_line_with_no_command_is_refused_and_one_with_a_command_is_not() {
        assert_eq!(
            TrustedArgv::after_separator(os(&["a", "b"])),
            Err(TrustedArgvError::NoSeparator)
        );
        assert_eq!(
            TrustedArgv::after_separator(os(&["a", "--"])),
            Err(TrustedArgvError::Empty)
        );
        assert_eq!(
            TrustedArgv::after_separator(os(&["a", "--", ""])),
            Err(TrustedArgvError::EmptyProgram)
        );
        assert!(TrustedArgv::after_separator(os(&["a", "--", "true"])).is_ok());
        // An empty argument after the program is an argument like any other.
        assert!(TrustedArgv::after_separator(os(&["a", "--", "true", ""])).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn a_word_that_is_not_utf8_is_refused_by_its_place() {
        use std::os::unix::ffi::OsStringExt as _;
        let mut args = os(&["a", "--", "printf"]);
        args.push(OsString::from_vec(vec![0x66, 0xff]));
        assert_eq!(
            TrustedArgv::after_separator(args),
            Err(TrustedArgvError::NotUtf8 { index: 1 })
        );
    }

    #[test]
    fn the_text_quotes_each_word_a_shell_would_split_or_expand() {
        let argv = TrustedArgv::new(
            [
                "cargo",
                "bench",
                "--bench=parse",
                "a b",
                "",
                "it's",
                "$HOME",
                "x;y",
            ]
            .map(str::to_string)
            .to_vec(),
        )
        .expect("a command");
        assert_eq!(
            argv.text(),
            "cargo bench --bench=parse 'a b' '' 'it'\\''s' '$HOME' 'x;y'"
        );
    }

    #[test]
    fn an_error_about_the_command_quotes_its_words_as_the_record_stores_them() {
        use crate::exec::{
            ExecError, Invocation, Limits, Programs, Runner as _, Stream, SystemRunner,
        };
        let words = |words: &[&str]| {
            TrustedArgv::new(words.iter().map(ToString::to_string).collect()).expect("a command")
        };
        // Joined unquoted, these two commands would read the same.
        let one = words(&["sh", "-c", "yes >&2"]);
        let other = words(&["sh", "-c", "yes", ">&2"]);
        assert_eq!(Invocation::trusted(&one).to_string(), "sh -c 'yes >&2'");
        assert_eq!(Invocation::trusted(&other).to_string(), "sh -c yes '>&2'");
        for argv in [&one, &other] {
            assert_eq!(Invocation::trusted(argv).to_string(), argv.text());
        }
        let error = ExecError::OutputTooLarge {
            command: Invocation::trusted(&one).to_string(),
            stream: Stream::Stderr,
            limit: 16,
            elapsed: std::time::Duration::from_millis(1500),
        };
        assert!(
            error
                .to_string()
                .starts_with("`sh -c 'yes >&2'` wrote more than 16 bytes"),
            "{error}"
        );
        // A real run of a program that cannot start names it the same way.
        let missing = words(&["./no such tool", "a b", ""]);
        let runner = SystemRunner::new(Programs::new(std::env::temp_dir()));
        let error = runner
            .run(&Invocation::trusted(&missing), &Limits::trusted())
            .expect_err("no such program");
        assert!(
            matches!(&error, ExecError::Spawn { command, .. } if *command == missing.text()),
            "{error:?}"
        );
        assert!(
            error
                .to_string()
                .starts_with("cannot start `'./no such tool' 'a b' ''`"),
            "{error}"
        );
        // The neighbour: a command of plain words reads as its words.
        let plain = words(&["cargo", "bench", "--bench=parse"]);
        assert_eq!(
            Invocation::trusted(&plain).to_string(),
            "cargo bench --bench=parse"
        );
    }
}
