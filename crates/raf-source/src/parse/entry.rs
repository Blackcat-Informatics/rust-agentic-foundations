// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The entry boundary of every parser worker.
//!
//! The native lexer keeps its source registry in thread-local storage. A host
//! thread-spawn callback runs on the new thread before the worker body and can
//! lex source there, leaving registrations behind. The pinned lexer offers no
//! read-only occupancy query, so the boundary lexes one fixed probe and reads
//! the file ordinal the registry assigns it: a registry that holds only its
//! dummy entry numbers the probe first. Any other ordinal refuses the worker
//! before it registers its own source; the probe's own registration remains in
//! the refused thread's registry. A clean registry is returned to its
//! dummy-only state, so the worker's source is registered first.

use proc_macro2::TokenStream;

use crate::error::{LoadError, LoadErrorKind};
use crate::path::RelPath;

/// The file name the registry gives the first source lexed on a fresh thread.
const FIRST_FILE: &str = "<parsed string 1>";

/// Claim the current thread's native source registry for one parser worker.
///
/// # Errors
/// Returns [`LoadErrorKind::Registry`] when a registration exists before the
/// worker's own. The refusal never clears or reuses foreign registrations. It
/// does leave the probe's own registration beside them, because the probe has
/// lexed before the occupancy is known; the refused thread ends without lexing
/// again.
pub(super) fn claim_registry(file: &RelPath) -> Result<(), LoadError> {
    #[cfg(test)]
    tests::host_callback(file);
    let first = "a"
        .parse::<TokenStream>()
        .ok()
        .and_then(|tokens| tokens.into_iter().next())
        .map_or_else(|| "<unreadable>".to_owned(), |token| token.span().file());
    if first != FIRST_FILE {
        return Err(LoadError::new(
            file,
            None,
            LoadErrorKind::Registry { first },
        ));
    }
    // Only the probe's own registration exists: drop it so the worker's source
    // is the first registration, as the admitted storage formulas assume.
    proc_macro2::extra::invalidate_current_thread_spans();
    Ok(())
}

#[cfg(test)]
pub(super) mod tests {
    use std::sync::Mutex;

    use super::{FIRST_FILE, TokenStream, claim_registry};
    use crate::error::LoadErrorKind;
    use crate::path::RelPath;

    /// A stand-in for a host spawn callback: it runs on the worker thread
    /// before the claim, for the one file it names.
    type Callbacks = Vec<(String, fn())>;

    static HOST: Mutex<Callbacks> = Mutex::new(Vec::new());

    pub(super) fn host_callback(file: &RelPath) {
        let name = file.to_string();
        let callback = HOST
            .lock()
            .expect("the host stand-in lock is never poisoned")
            .iter()
            .find(|(registered, _)| *registered == name)
            .map(|(_, callback)| *callback);
        if let Some(callback) = callback {
            callback();
        }
    }

    pub(in crate::parse) fn with_host<R>(
        file: &RelPath,
        callback: fn(),
        run: impl FnOnce() -> R,
    ) -> R {
        let name = file.to_string();
        HOST.lock()
            .expect("the host stand-in lock is never poisoned")
            .push((name.clone(), callback));
        let result = run();
        HOST.lock()
            .expect("the host stand-in lock is never poisoned")
            .retain(|(registered, _)| *registered != name);
        result
    }

    pub(in crate::parse) fn lex_foreign() {
        drop("fn host() {}".parse::<TokenStream>().expect("lexes"));
    }

    #[test]
    fn a_worker_whose_registry_a_host_callback_populated_is_refused_before_it_registers() {
        let file = RelPath::new("src/occupied_registry.rs").expect("path");
        let error = with_host(&file, lex_foreign, || {
            crate::parse_source(&file, "pub fn f() {}\n", |_| ())
        })
        .expect_err("an occupied registry refuses the worker");
        assert!(
            matches!(&error.kind, LoadErrorKind::Registry { first } if first != FIRST_FILE),
            "{error}"
        );
        let text = error.to_string();
        assert!(text.starts_with("src/occupied_registry.rs: "), "{text}");
        assert!(text.contains("spawn callback"), "{text}");
    }

    #[test]
    fn a_worker_with_a_fresh_registry_parses_and_registers_its_source_first() {
        let file = RelPath::new("src/fresh_registry.rs").expect("path");
        let first = with_host(
            &file,
            || {},
            || {
                crate::parse_source_with_tokens(&file, "pub fn f() {}\n", |_, tokens| {
                    tokens.into_iter().next().expect("a token").span().file()
                })
            },
        )
        .expect("a fresh registry parses");
        assert_eq!(first, FIRST_FILE);
    }

    #[test]
    fn claiming_a_populated_thread_refuses_and_a_fresh_thread_is_claimed() {
        let file = RelPath::new("src/claim.rs").expect("path");
        std::thread::scope(|scope| {
            let fresh = scope.spawn(|| claim_registry(&file));
            assert!(fresh.join().expect("joined").is_ok());
            let used = scope.spawn(|| {
                lex_foreign();
                claim_registry(&file)
            });
            let refused = used.join().expect("joined").expect_err("occupied");
            assert!(matches!(refused.kind, LoadErrorKind::Registry { .. }));
        });
    }
}
