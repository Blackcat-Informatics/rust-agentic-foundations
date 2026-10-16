// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The digest of a record input: the SHA-256 of a canonical encoding of the
//! text the input names.
//!
//! An evidence record and an assessment keep one digest for each input they
//! were made against, and an input whose digest now differs has moved. The
//! three inputs are an authored section ([`Digest::section`]), the source
//! text of a symbol ([`Digest::symbol`]) and the text of a record
//! ([`Digest::record`]).
//!
//! The hashed bytes are one or more fields, each written as
//!
//! ```text
//! <kind> NUL <name> NUL <length> NUL <bytes>
//! ```
//!
//! where `kind` is `section`, `symbol` or `record`, `name` names the input
//! (the section slug, the symbol path, or nothing for a record), `length` is
//! the number of bytes in decimal ASCII and `bytes` is the text. The kind
//! separates the domains, so a section and a symbol with the same text have
//! different digests, and the length ends each field. The kind and the name
//! end at their NUL, and no kind or name holds one (a kind is one of the
//! three words, a name is a section slug, a resolved symbol path or empty), so
//! no two different sequences of fields encode to the same bytes. Every text
//! has its CRLF line endings replaced by LF exactly once, in one pass, before
//! it is hashed, so a checkout that converts line endings does not move a
//! digest; a lone CR is text and is kept, so `x\r\r\ny` is hashed with one CR
//! left before its LF and differs from both `x\r\ny` and `x\ny`. The
//! digest is a pure function of the text: equal on every run and every
//! platform.
//!
//! SHA-256 is the function of FIPS 180-4, implemented here with no
//! dependency. The digest detects change; it is not a secret, and its
//! comparison is not constant-time.

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use raf_okf::{Body, CapabilitySection};
use raf_source::{Item, ResolveError, SymbolKind, Workspace};

use crate::lifecycle::section_digest;

/// The SHA-256 digest of a record input, written as 64 lowercase hexadecimal
/// digits.
///
/// ```rust
/// use raf_workplan::digest::Digest;
///
/// let digest = Digest::record("---\ntype: Coverage\n---\n");
/// let text = digest.to_string();
/// assert_eq!(text.len(), 64);
/// assert_eq!(text.parse::<Digest>()?, digest);
/// // CRLF line endings are read as LF.
/// assert_eq!(Digest::record("---\r\ntype: Coverage\r\n---\r\n"), digest);
/// # Ok::<(), raf_workplan::digest::ParseDigestError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest([u8; 32]);

impl Digest {
    /// The digest of the authored text of the section `section` of `body`,
    /// or `None` when the body has no such section.
    ///
    /// The text is the one [`section_digest`] keeps: the heading and the
    /// content of the section, with the content of each generated block left
    /// out and CRLF read as LF (the one pass of the module), so syncing a
    /// generated block does not move the digest and an edit of the prose
    /// does. The field is `section NUL <section slug> NUL <length> NUL <text>`.
    #[must_use]
    pub fn section(body: &Body, section: CapabilitySection) -> Option<Self> {
        // `section_digest` has replaced CRLF already; the text is hashed as it is.
        let authored = section_digest(body, section)?;
        let mut hasher = Sha256::new();
        field(&mut hasher, "section", section.slug(), authored.text());
        Some(Self(hasher.finish()))
    }

    /// The digest of the source text of the symbol `path`, resolved through
    /// `source` as a binding is.
    ///
    /// The text of a definition is its whole span in its file, attributes and
    /// documentation included, with CRLF read as LF. A path with more than one
    /// definition (`#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()`)
    /// hashes each once, in the order of file and position; a definition the
    /// index reaches under several module paths, because its file is read by
    /// more than one target, has one source key and counts once. Each
    /// definition is the field `symbol NUL <path> NUL <length> NUL <text>`.
    ///
    /// A module stands for its whole content, as it does in coverage: after
    /// the field of its own span (an inline module's braces, or the
    /// declaration `mod name;`), each file of the module and of the modules
    /// below it in the same target is one more field with the file's whole
    /// text, the fields in the order of file and position. Editing any item inside a bound
    /// module, in its own file or in a child module's file, moves the digest;
    /// editing a sibling module does not. A crate root stands for nothing
    /// below it, so only its own span is hashed.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::digest::Digest;
    ///
    /// let load = |lib: &str| {
    ///     Workspace::load(
    ///         &MemoryTree::from([
    ///             ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///             ("src/lib.rs", lib),
    ///         ]),
    ///         &["."],
    ///     )
    /// };
    /// let before = load("pub fn other() {}\npub fn run() {}\n")?;
    /// let after = load("pub fn other() { let _ = 1; }\npub fn run() {}\n")?;
    /// // Editing another item leaves the digest of `run` as it was.
    /// assert_eq!(Digest::symbol(&before, "demo::run")?, Digest::symbol(&after, "demo::run")?);
    /// assert_ne!(Digest::symbol(&before, "demo::other")?, Digest::symbol(&after, "demo::other")?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`SymbolDigestError::Resolve`] when the path does not resolve to one
    /// symbol, and [`SymbolDigestError::Source`] when the index holds no text
    /// for a definition's span.
    pub fn symbol(source: &Workspace, path: &str) -> Result<Self, SymbolDigestError> {
        let sites = symbol_sites(source, path)?;
        Ok(symbol_of(path, sites))
    }

    /// The digest of the text of a record file, with CRLF read as LF: the
    /// field `record NUL NUL <length> NUL <text>`.
    #[must_use]
    pub fn record(text: &str) -> Self {
        let mut hasher = Sha256::new();
        field(&mut hasher, "record", "", &lf(text));
        Self(hasher.finish())
    }

    /// The 32 bytes of the digest.
    #[must_use]
    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for Digest {
    /// The 64 lowercase hexadecimal digits, the form a record stores.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Digest({self})")
    }
}

impl FromStr for Digest {
    type Err = ParseDigestError;

    /// Read the form [`Digest`] displays: exactly 64 lowercase hexadecimal
    /// digits, the form the profile accepts in a record.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let refuse = || ParseDigestError {
            found: text.to_string(),
        };
        let digits = text.as_bytes();
        if digits.len() != 64 {
            return Err(refuse());
        }
        let nibble = |digit: u8| match digit {
            b'0'..=b'9' => Some(digit - b'0'),
            b'a'..=b'f' => Some(digit - b'a' + 10),
            _ => None,
        };
        let mut bytes = [0_u8; 32];
        for (byte, [high, low]) in bytes.iter_mut().zip(digits.as_chunks::<2>().0) {
            let (Some(high), Some(low)) = (nibble(*high), nibble(*low)) else {
                return Err(refuse());
            };
            *byte = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

/// A text that is not a digest: not 64 lowercase hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDigestError {
    found: String,
}

impl fmt::Display for ParseDigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is not a digest; a digest is 64 lowercase hexadecimal digits",
            self.found
        )
    }
}

impl Error for ParseDigestError {}

/// Why a symbol has no digest.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SymbolDigestError {
    /// The path does not resolve to one symbol.
    Resolve {
        /// The path as given.
        path: String,
        /// What the source index said.
        error: ResolveError,
    },
    /// The index holds no text for the span of a definition of the path.
    Source {
        /// The path as given.
        path: String,
    },
}

impl fmt::Display for SymbolDigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolve { path, error } => {
                write!(f, "`{path}` has no digest: {error}")
            }
            Self::Source { path } => write!(
                f,
                "`{path}` has no digest: the source index holds no text for a definition"
            ),
        }
    }
}

impl Error for SymbolDigestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolve { error, .. } => Some(error),
            Self::Source { .. } => None,
        }
    }
}

/// A text of a symbol: its key (file, start offset, and whether it is a whole
/// file of a module rather than an item's span) and the text.
type Site<'w> = ((&'w str, usize, bool), &'w str);

/// The definitions of the symbol `path`, each with its source key (file and
/// start offset) and its source text, in no particular order and possibly
/// with a definition listed twice; [`symbol_of`] and [`symbol_texts`] order
/// and de-duplicate them.
fn symbol_sites<'w>(source: &'w Workspace, path: &str) -> Result<Vec<Site<'w>>, SymbolDigestError> {
    let resolution = source
        .resolve(path)
        .map_err(|error| SymbolDigestError::Resolve {
            path: path.to_string(),
            error,
        })?;
    let mut sites = Vec::new();
    for id in resolution.sites {
        let missing = || SymbolDigestError::Source {
            path: path.to_string(),
        };
        let item = source.item(id).ok_or_else(missing)?;
        let text = source
            .source(&item.file)
            .and_then(|text| text.get(item.span.start_byte..item.span.end_byte))
            .ok_or_else(missing)?;
        let (file, start) = item.source_key();
        sites.push(((file.as_str(), start, false), text));
        if item.kind == SymbolKind::Mod {
            module_files(source, item, &mut sites);
        }
    }
    Ok(sites)
}

/// Add the whole text of each file of the module `module` and of the modules
/// below it, in the module's target: the content a module binding stands
/// for. A crate root stands for nothing below it, as in coverage, so only its
/// own span is hashed.
fn module_files<'w>(source: &'w Workspace, module: &Item, sites: &mut Vec<Site<'w>>) {
    let is_root = source
        .target(module.target)
        .is_some_and(|target| *module.path == *target.crate_path);
    if is_root {
        return;
    }
    let below = format!("{}::", module.path);
    for file in source.files() {
        let reached = file.reached.iter().any(|reach| {
            reach.target == module.target
                && (*reach.module_path == *module.path || reach.module_path.starts_with(&below))
        });
        if reached {
            sites.push(((file.path.as_str(), 0, true), file.text.as_str()));
        }
    }
}

/// The source text of each definition of the symbol `path`, as
/// [`Digest::symbol`] hashes it: once per source key, in key order, with CRLF
/// read as LF. Each entry is the file and the text.
pub(crate) fn symbol_texts(
    source: &Workspace,
    path: &str,
) -> Result<Vec<(String, String)>, SymbolDigestError> {
    let mut sites = symbol_sites(source, path)?;
    sites.sort_by(|a, b| a.0.cmp(&b.0));
    sites.dedup_by(|a, b| a.0 == b.0);
    Ok(sites
        .into_iter()
        .map(|((file, _, _), text)| (file.to_string(), lf(text)))
        .collect())
}

/// The digest of the symbol `path` from the texts of its definitions, each
/// with its source key (file and start offset): one field per source key, in
/// key order, so a definition listed twice counts once.
fn symbol_of(path: &str, mut sites: Vec<Site<'_>>) -> Digest {
    sites.sort_by(|a, b| a.0.cmp(&b.0));
    sites.dedup_by(|a, b| a.0 == b.0);
    let mut hasher = Sha256::new();
    for (_, text) in sites {
        field(&mut hasher, "symbol", path, &lf(text));
    }
    Digest(hasher.finish())
}

/// `text` with each CRLF replaced by LF, in one pass.
fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Feed one field `kind NUL name NUL length NUL text` to `hasher`. The text
/// is hashed as given: the callers have replaced its CRLF line endings once.
fn field(hasher: &mut Sha256, kind: &str, name: &str, text: &str) {
    hasher.update(kind.as_bytes());
    hasher.update(&[0]);
    hasher.update(name.as_bytes());
    hasher.update(&[0]);
    hasher.update(text.len().to_string().as_bytes());
    hasher.update(&[0]);
    hasher.update(text.as_bytes());
}

/// The initial hash value of SHA-256 (FIPS 180-4, 5.3.3).
const INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// The round constants of SHA-256 (FIPS 180-4, 4.2.2).
const ROUND: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// SHA-256 over a stream of bytes (FIPS 180-4, 6.2).
pub(crate) struct Sha256 {
    state: [u32; 8],
    /// The bytes of the current block not yet compressed.
    block: [u8; 64],
    /// How many bytes of `block` are filled.
    filled: usize,
    /// The number of bytes fed so far.
    length: u64,
}

impl Sha256 {
    pub(crate) fn new() -> Self {
        Self {
            state: INITIAL,
            block: [0; 64],
            filled: 0,
            length: 0,
        }
    }

    pub(crate) fn update(&mut self, mut bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        while !bytes.is_empty() {
            let take = (64 - self.filled).min(bytes.len());
            self.block[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            bytes = &bytes[take..];
            if self.filled == 64 {
                compress(&mut self.state, &self.block);
                self.filled = 0;
            }
        }
    }

    /// Pad the message (FIPS 180-4, 5.1.1) and return the hash.
    pub(crate) fn finish(mut self) -> [u8; 32] {
        let bits = self.length.wrapping_mul(8);
        self.block[self.filled] = 0x80;
        self.block[self.filled + 1..].fill(0);
        if self.filled >= 56 {
            compress(&mut self.state, &self.block);
            self.block = [0; 64];
        }
        self.block[56..].copy_from_slice(&bits.to_be_bytes());
        compress(&mut self.state, &self.block);
        let mut out = [0_u8; 32];
        for (chunk, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(self.state) {
            *chunk = word.to_be_bytes();
        }
        out
    }
}

/// The compression function on one 512-bit block (FIPS 180-4, 6.2.2).
#[allow(
    clippy::many_single_char_names,
    reason = "the working variables keep the names a to h of FIPS 180-4, 6.2.2"
)]
fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut schedule = [0_u32; 64];
    for (word, chunk) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*chunk);
    }
    for t in 16..64 {
        let s0 = schedule[t - 15].rotate_right(7)
            ^ schedule[t - 15].rotate_right(18)
            ^ (schedule[t - 15] >> 3);
        let s1 = schedule[t - 2].rotate_right(17)
            ^ schedule[t - 2].rotate_right(19)
            ^ (schedule[t - 2] >> 10);
        schedule[t] = schedule[t - 16]
            .wrapping_add(s0)
            .wrapping_add(schedule[t - 7])
            .wrapping_add(s1);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for (constant, word) in ROUND.iter().zip(schedule) {
        let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let t1 = h
            .wrapping_add(big_s1)
            .wrapping_add(choose)
            .wrapping_add(*constant)
            .wrapping_add(word);
        let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let t2 = big_s0.wrapping_add(majority);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    for (word, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *word = word.wrapping_add(value);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::{Digest, Sha256, field, symbol_of};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
    }

    fn sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex(&hasher.finish())
    }

    /// The test vectors of the NIST examples for SHA-256 (FIPS 180-4).
    #[test]
    fn sha256_gives_the_standard_test_vectors() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            sha256(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    /// The padding boundaries: 55 bytes fit one block, 56 need two, and a
    /// message fed in pieces hashes as it does whole.
    #[test]
    fn sha256_is_the_same_over_block_boundaries_and_pieces() {
        let message: Vec<u8> = (0..=255_u8).cycle().take(1000).collect();
        for length in [0, 1, 55, 56, 63, 64, 65, 119, 120, 128, 1000] {
            let whole = sha256(&message[..length]);
            for piece in [1, 3, 64, 100] {
                let mut hasher = Sha256::new();
                for chunk in message[..length].chunks(piece) {
                    hasher.update(chunk);
                }
                assert_eq!(
                    hex(&hasher.finish()),
                    whole,
                    "{length} in pieces of {piece}"
                );
            }
        }
        // 56 bytes leave no room for the length, so the padding takes a
        // second block (checked against `sha256sum`).
        assert_eq!(
            sha256(&[b'a'; 56]),
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a"
        );
    }

    fn digest_of(fields: &[(&str, &str, &str)]) -> Digest {
        let mut hasher = Sha256::new();
        for (kind, name, text) in fields {
            field(&mut hasher, kind, name, text);
        }
        Digest(hasher.finish())
    }

    /// The kind separates the domains: equal names and equal bytes give
    /// different digests in different kinds.
    #[test]
    fn fields_of_different_kinds_with_equal_bytes_differ() {
        let text = "pub fn run() {}\n";
        let section = digest_of(&[("section", "run", text)]);
        let symbol = digest_of(&[("symbol", "run", text)]);
        let record = digest_of(&[("record", "run", text)]);
        assert_ne!(section, symbol);
        assert_ne!(symbol, record);
        assert_ne!(section, record);
    }

    /// The length ends each field, so moving bytes between the name and the
    /// text, or between two fields, changes the digest.
    #[test]
    fn the_encoding_is_unambiguous_across_names_texts_and_fields() {
        assert_ne!(
            digest_of(&[("symbol", "a", "bc")]),
            digest_of(&[("symbol", "ab", "c")])
        );
        assert_ne!(
            digest_of(&[("symbol", "p", "ab"), ("symbol", "p", "c")]),
            digest_of(&[("symbol", "p", "a"), ("symbol", "p", "bc")])
        );
        assert_ne!(
            digest_of(&[("symbol", "p", "abc")]),
            digest_of(&[("symbol", "p", "ab"), ("symbol", "p", "c")])
        );
    }

    /// A definition listed twice under one source key counts once, and the
    /// order the definitions are listed in does not matter.
    #[test]
    fn symbol_definitions_are_deduplicated_by_source_key_and_ordered() {
        let one = symbol_of("demo::g", vec![(("src/x.rs", 0, false), "pub fn g() {}")]);
        let twice = symbol_of(
            "demo::g",
            vec![
                (("src/x.rs", 0, false), "pub fn g() {}"),
                (("src/x.rs", 0, false), "pub fn g() {}"),
            ],
        );
        assert_eq!(one, twice);
        let a = (("src/lib.rs", 0, false), "#[cfg(unix)]\npub fn g() {}");
        let b = (
            ("src/lib.rs", 40, false),
            "#[cfg(not(unix))]\npub fn g() {}",
        );
        assert_eq!(
            symbol_of("demo::g", vec![a, b]),
            symbol_of("demo::g", vec![b, a, b])
        );
        assert_ne!(
            symbol_of("demo::g", vec![a, b]),
            symbol_of("demo::g", vec![a])
        );
    }
}
