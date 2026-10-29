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
//! SHA-256 uses the same streaming implementation as `PurRDF`; its public
//! content-store API supplies one-shot hashing, not a streaming state. The digest detects change; it is not a secret, and its
//! comparison is not constant-time.
//!
//! Semantic identifiers and diagram asset identities separately use unframed
//! BLAKE3 over exact bytes, composed with the native lowercase hex renderer.
//! That internal identity path applies no record-input framing or line-ending
//! normalization and does not change the SHA-256 record-input contract.

use sha2::{Digest as _, Sha256};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use raf_okf::{Body, CapabilitySection};
use raf_source::{Item, ResolveError, SymbolKind, Workspace};

use crate::lifecycle::section_digest;

/// Exact native BLAKE3 identity, separate from framed record-input digests.
pub(crate) fn semantic_hash(text: &str) -> String {
    purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(text.as_bytes()).as_bytes()).to_string()
}

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
        Some(Self(hasher.finalize().into()))
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

    /// The source owned by one indexed definition for a code review.
    ///
    /// Separately indexed nested definitions are removed before hashing, so a
    /// changed method does not also invalidate its trait or enclosing module.
    /// Container attributes, imports, impl headers and other unindexed text
    /// remain covered. An external module also owns the residual text of its
    /// exact module files, without any descendant module files. Source offsets
    /// select text but are never hashed. CRLF is normalized once to LF.
    ///
    /// This uses a separate digest domain from aggregate binding digests;
    /// existing evidence and assessment inputs retain their original semantics.
    ///
    /// # Errors
    /// Returns [`SymbolDigestError::Source`] if an indexed span has no source.
    pub fn review_item(source: &Workspace, item: &Item) -> Result<Self, SymbolDigestError> {
        let mut hasher = Sha256::new();
        let text = review_residual(
            source,
            item,
            &item.file,
            item.span.start_byte,
            item.span.end_byte,
        )?;
        field(&mut hasher, "review-item", item.path.as_str(), &lf(&text));
        if item.kind == SymbolKind::Mod {
            for file in source.files().filter(|file| {
                file.path != item.file
                    && file
                        .reached
                        .iter()
                        .any(|reach| reach.target == item.target && reach.module_path == item.path)
            }) {
                let text = review_residual(source, item, &file.path, 0, file.text.len())?;
                field(
                    &mut hasher,
                    "review-module-file",
                    file.path.as_str(),
                    &lf(&text),
                );
            }
        }
        Ok(Self(hasher.finalize().into()))
    }

    /// The review digest of every definition of one canonical item path.
    ///
    /// Conditional definitions remain distinct source sites. A physical site
    /// reached through multiple routes is counted once; file and namespace
    /// distinguish definitions, while byte offsets only order equal-path sites.
    ///
    /// # Errors
    /// Returns [`SymbolDigestError::Source`] for an absent item or missing text.
    pub fn review_symbol(source: &Workspace, path: &str) -> Result<Self, SymbolDigestError> {
        let mut items: Vec<_> = source
            .items_under(path)
            .filter(|item| item.path.as_str() == path)
            .collect();
        purrdf_lex::walk::try_sort_unstable_by(&mut items, |left, right| {
            Ok::<_, std::convert::Infallible>(
                (&left.file, left.namespace, left.span.start_byte).cmp(&(
                    &right.file,
                    right.namespace,
                    right.span.start_byte,
                )),
            )
        })
        .unwrap_or_else(|never| match never {});
        purrdf_lex::walk::try_dedup_by(&mut items, |left, right| {
            Ok::<_, std::convert::Infallible>(
                left.source_key() == right.source_key() && left.namespace == right.namespace,
            )
        })
        .unwrap_or_else(|never| match never {});
        if items.is_empty() {
            return Err(SymbolDigestError::Source {
                path: path.to_string(),
            });
        }
        let mut hasher = Sha256::new();
        for item in items {
            let name = format!("{}:{}:{}", item.namespace, item.file, item.path);
            let digest = Self::review_item(source, item)?;
            field(&mut hasher, "review-definition", &name, &digest.to_string());
        }
        Ok(Self(hasher.finalize().into()))
    }

    /// The digest of the text of a record file, with CRLF read as LF: the
    /// field `record NUL NUL <length> NUL <text>`.
    #[must_use]
    pub fn record(text: &str) -> Self {
        let mut hasher = Sha256::new();
        field(&mut hasher, "record", "", &lf(text));
        Self(hasher.finalize().into())
    }

    /// The 32 bytes of the digest.
    #[must_use]
    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Keep a container's own text while removing maximal indexed child spans.
fn review_residual(
    source: &Workspace,
    owner: &Item,
    file: &raf_source::RelPath,
    start: usize,
    end: usize,
) -> Result<String, SymbolDigestError> {
    let missing = || SymbolDigestError::Source {
        path: owner.path.to_string(),
    };
    let text = source.source(file).ok_or_else(missing)?;
    text.get(start..end).ok_or_else(missing)?;
    let is_root = |item: &Item| {
        source
            .target(item.target)
            .is_some_and(|target| item.path == target.crate_path)
    };
    let owner_is_root = is_root(owner);
    let mut children: Vec<_> = source
        .items_in(file)
        .filter(|child| {
            child.target == owner.target
                && child.id != owner.id
                && !is_root(child)
                && (owner_is_root || child.source_key() != owner.source_key())
        })
        .filter_map(|child| {
            let span = &child.span;
            (span.start_byte >= start && span.end_byte <= end)
                .then_some((span.start_byte, span.end_byte))
        })
        .collect();
    purrdf_lex::walk::try_sort_unstable_by(&mut children, |a, b| {
        Ok::<_, std::convert::Infallible>(a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)))
    })
    .unwrap_or_else(|never| match never {});
    let mut kept = String::new();
    let mut cursor = start;
    for (first, last) in children {
        if first < cursor {
            continue;
        }
        kept.push_str(text.get(cursor..first).ok_or_else(missing)?);
        cursor = last;
    }
    kept.push_str(text.get(cursor..end).ok_or_else(missing)?);
    Ok(kept)
}

impl fmt::Display for Digest {
    /// The 64 lowercase hexadecimal digits, the form a record stores.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&purrdf_hash::hex::Lower(&self.0), f)
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
        purrdf_hash::hex::decode_32_canonical(text)
            .map(Self)
            .ok_or_else(refuse)
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

impl SymbolDigestError {
    pub(crate) fn is_query_refusal(&self) -> bool {
        matches!(
            self,
            Self::Resolve {
                error: ResolveError::Budget { .. } | ResolveError::Storage { .. },
                ..
            }
        )
    }
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
            sites.extend(
                module_files(source, item)
                    .into_iter()
                    .map(|(file, text)| ((file, 0, true), text)),
            );
        }
    }
    Ok(sites)
}

/// The path and the whole text of each file of the module `module` and of
/// the modules below it, in the module's target, in the order of the files
/// of the index: the content a module binding stands for. A crate root
/// stands for nothing below it, as in coverage, so it has no file here and
/// only its own span is hashed. The history of a module binding reads the
/// log of each of these files.
pub(crate) fn module_files<'w>(source: &'w Workspace, module: &Item) -> Vec<(&'w str, &'w str)> {
    let is_root = source
        .target(module.target)
        .is_some_and(|target| *module.path == *target.crate_path);
    if is_root {
        return Vec::new();
    }
    let below = format!("{}::", module.path);
    source
        .files()
        .filter(|file| {
            file.reached.iter().any(|reach| {
                reach.target == module.target
                    && (*reach.module_path == *module.path || reach.module_path.starts_with(&below))
            })
        })
        .map(|file| (file.path.as_str(), file.text.as_str()))
        .collect()
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
    Digest(hasher.finalize().into())
}

/// `text` with each CRLF replaced by LF, in one pass.
fn lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Feed one field `kind NUL name NUL length NUL text` to `hasher`. The text
/// is hashed as given: the callers have replaced its CRLF line endings once.
fn field(hasher: &mut Sha256, kind: &str, name: &str, text: &str) {
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(name.as_bytes());
    hasher.update([0]);
    hasher.update(text.len().to_string().as_bytes());
    hasher.update([0]);
    hasher.update(text.as_bytes());
}

/// Identity framing and the shared SHA-256 implementation's known vectors.
#[cfg(test)]
mod tests {
    use sha2::Digest as _;

    use super::{Digest, Sha256, field, symbol_of};

    /// A review owns each leaf once, including a first declaration at byte zero.
    #[test]
    fn review_currency_changes_only_the_edited_definition_and_keeps_container_contracts() {
        let load = |text: &str| {
            raf_source::Workspace::load(
                &raf_source::MemoryTree::from([
                    (
                        "Cargo.toml",
                        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
                    ),
                    ("src/lib.rs", text),
                ]),
                &["."],
            )
            .expect("the review fixture is valid source")
        };
        let original = "pub fn first() { let _ = 1; }\npub mod nested { pub fn second() {} }\npub trait Service { fn run(&self) { let _ = 2; } }\n";
        let before = load(original);
        for (old, new, expected) in [
            ("let _ = 1", "let _ = 10", "demo::first"),
            ("let _ = 2", "let _ = 20", "demo::Service::run"),
            ("pub mod nested", "pub(crate) mod nested", "demo::nested"),
        ] {
            let after = load(&original.replace(old, new));
            let changed: Vec<_> = before
                .items()
                .filter_map(|item| {
                    let other = after
                        .items()
                        .find(|candidate| candidate.path == item.path)
                        .expect("the edit preserves every item");
                    let left = Digest::review_item(&before, item).expect("indexed source");
                    let right = Digest::review_item(&after, other).expect("indexed source");
                    (left != right).then(|| item.path.to_string())
                })
                .collect();
            assert_eq!(changed, [expected]);
        }
        let converted = load(&original.replace('\n', "\r\n"));
        for item in before.items() {
            let other = converted
                .items()
                .find(|candidate| candidate.path == item.path)
                .expect("line endings preserve items");
            assert_eq!(
                Digest::review_item(&before, item).expect("indexed source"),
                Digest::review_item(&converted, other).expect("indexed source"),
            );
        }
    }

    fn hex(bytes: &[u8]) -> String {
        purrdf_hash::hex::encode(bytes)
    }

    fn sha256(bytes: &[u8]) -> String {
        purrdf_core::content_store::ContentDigest::of(bytes).to_hex()
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
                    hex(&hasher.finalize()),
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
        Digest(hasher.finalize().into())
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
