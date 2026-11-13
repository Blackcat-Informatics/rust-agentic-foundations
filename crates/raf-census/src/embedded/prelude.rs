// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A declared language profile, never an observed compiler or sysroot identity.

use raf_source::{SourceFile, Workspace};

use super::names;
use super::resource::{Meter, Refusal};

/// Standard-prelude value rules of the versioned fragment syntax model.
/// The edition is retained even where its fixed variant inventory agrees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Prelude {
    edition: u16,
    disabled: bool,
    no_std: bool,
}

impl Prelude {
    /// Declared edition only. Ancestor policy is joined by `context::for_file`
    /// before this profile is used by the workspace analysis.
    pub(super) fn declared_for_file(
        workspace: &Workspace,
        file: &SourceFile,
        meter: &Meter,
    ) -> Result<Self, Refusal> {
        meter.work(file.path.as_str().len())?;
        let owned = workspace
            .file(&file.path)
            .filter(|owned| std::ptr::eq(*owned, file))
            .ok_or(Refusal::Invariant(
                "prelude context is not the workspace's actual source file",
            ))?;
        let mut selected = None;
        for reach in &owned.reached {
            meter.work(1)?;
            workspace
                .target(reach.target)
                .ok_or(Refusal::Invariant("source reach has no actual target"))?;
            let edition = workspace
                .target_edition(reach.target)
                .ok_or(Refusal::Invariant("source target has no declared edition"))?;
            if !matches!(edition, 2015 | 2018 | 2021 | 2024) {
                return Err(Refusal::Unsupported(
                    "declared Rust edition has no embedded syntax/prelude profile",
                ));
            }
            let profile = Self {
                edition,
                disabled: false,
                no_std: false,
            };
            if selected.is_some_and(|known| known != profile) {
                return Err(Refusal::Unsupported(
                    "source reaches disagree on their declared prelude profile",
                ));
            }
            selected = Some(profile);
        }
        selected.ok_or(Refusal::Invariant("source file has no actual target reach"))
    }

    /// Only these standard variant imports can block a bare identifier pattern.
    /// The other standard imports are traits, functions, macros or types with
    /// no available value constructor. Explicit source scopes remain separate.
    pub(super) fn fixed(name: &str, meter: &Meter) -> Result<bool, Refusal> {
        for fixed in ["None", "Some", "Err", "Ok"] {
            if names::same(name, fixed, meter)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) const fn edition(self) -> u16 {
        self.edition
    }

    pub(super) const fn flags(self) -> (bool, bool) {
        (self.disabled, self.no_std)
    }

    pub(super) const fn with_flags(self, disabled: bool, no_std: bool) -> Self {
        Self {
            disabled,
            no_std,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedded::scopes::{PatternMeaning, Scopes};
    use raf_source::MemoryTree;

    #[test]
    fn actual_default_inherited_and_unsupported_editions_control_the_declared_profile() {
        for (declaration, expected) in [
            ("", Some(2015)),
            ("edition = \"2024\"\n", Some(2024)),
            ("edition = \"9999\"\n", None),
        ] {
            let manifest =
                format!("[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n{declaration}");
            let tree = MemoryTree::from([
                ("Cargo.toml", manifest.as_str()),
                ("src/lib.rs", "fn helper() {}"),
            ]);
            let workspace = Workspace::load(&tree, &["."]).unwrap();
            let file = workspace.files().next().unwrap();
            let meter = Meter::new(100_000, 100_000);
            let result = Prelude::declared_for_file(&workspace, file, &meter);
            match expected {
                Some(edition) => assert_eq!(result.unwrap().edition(), edition),
                None => assert!(matches!(result, Err(Refusal::Unsupported(_)))),
            }
        }
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"member\"]\n[workspace.package]\nedition = \"2021\"\n",
            ),
            (
                "member/Cargo.toml",
                "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition.workspace = true\n",
            ),
            ("member/src/lib.rs", "fn helper() {}"),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let file = workspace.files().next().unwrap();
        assert_eq!(
            Prelude::declared_for_file(&workspace, file, &Meter::new(100_000, 100_000))
                .unwrap()
                .edition(),
            2021
        );
    }

    #[test]
    fn declared_standard_names_stay_fixed_without_mistaking_type_or_macro_names_for_values() {
        let source = "fn helper(input: i32) -> i32 { let café = input; cafe\u{301} + 1 }";
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
            ),
            ("src/lib.rs", source),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let file = workspace.files().next().unwrap();
        let profile =
            Prelude::declared_for_file(&workspace, file, &Meter::new(100_000, 100_000)).unwrap();
        raf_source::parse_source(&file.path, source, |syntax| {
            let meter = Meter::new(100_000, 100_000);
            let coordinates = crate::embedded::coordinates::Coordinates::new(
                source,
                syntax.shebang.as_deref(),
                &meter,
            )
            .unwrap();
            let mut scopes = Scopes::with_prelude(source, &meter, profile, &coordinates);
            scopes
                .enter_module(&syntax.items, &syntax.attrs, true)
                .unwrap();
            for name in ["None", "Some", "Err", "Ok"] {
                assert_eq!(scopes.pattern(name).unwrap(), PatternMeaning::Fixed);
            }
            for name in [
                "ordinary", "Iterator", "Box", "Vec", "String", "assert", "drop",
            ] {
                assert_eq!(scopes.pattern(name).unwrap(), PatternMeaning::Binding);
            }
            let syn::Item::Fn(helper) = &syntax.items[0] else {
                panic!("source helper")
            };
            let facts = crate::embedded::collect::function(
                source,
                &helper.sig,
                &helper.block,
                &mut scopes,
                &meter,
            )
            .unwrap();
            let declared = facts
                .annotations
                .iter()
                .find(|annotation| {
                    !annotation.marker
                        && annotation.occurrence.event == crate::embedded::proof::Event::Declaration
                })
                .unwrap();
            let used = facts
                .annotations
                .iter()
                .find(|annotation| {
                    !annotation.marker
                        && source.get(annotation.span.start..annotation.span.end)
                            == Some("cafe\u{301}")
                })
                .unwrap();
            assert_eq!(declared.occurrence.binding, used.occurrence.binding);
        })
        .unwrap();
        let foreign = Workspace::load(&tree, &["."]).unwrap();
        assert!(matches!(
            Prelude::declared_for_file(&foreign, file, &Meter::new(100_000, 100_000)),
            Err(Refusal::Invariant(_))
        ));
    }

    #[test]
    fn disabled_preludes_have_no_fixed_variants_and_custom_or_malformed_flags_refuse() {
        for (attribute, admitted) in [
            ("#![no_implicit_prelude]", true),
            ("#![no_std]", true),
            ("#![no_core]", false),
            ("#![prelude_import]", false),
            ("#![no_std = true]", false),
        ] {
            let source = format!("{attribute}\nfn helper() {{}}");
            let tree = MemoryTree::from([
                (
                    "Cargo.toml",
                    "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
                ),
                ("src/lib.rs", source.as_str()),
            ]);
            let workspace = Workspace::load(&tree, &["."]).unwrap();
            let file = workspace.files().next().unwrap();
            let profile =
                Prelude::declared_for_file(&workspace, file, &Meter::new(100_000, 100_000))
                    .unwrap();
            raf_source::parse_source(&file.path, &source, |syntax| {
                let meter = Meter::new(100_000, 100_000);
                let coordinates = crate::embedded::coordinates::Coordinates::new(
                    &source,
                    syntax.shebang.as_deref(),
                    &meter,
                )
                .unwrap();
                let mut scopes = Scopes::with_prelude(&source, &meter, profile, &coordinates);
                let result = scopes.enter_module(&syntax.items, &syntax.attrs, true);
                if admitted {
                    result.unwrap();
                    let expected = if attribute == "#![no_implicit_prelude]" {
                        PatternMeaning::Binding
                    } else {
                        PatternMeaning::Fixed
                    };
                    assert_eq!(scopes.pattern("None").unwrap(), expected);
                } else {
                    assert!(matches!(result, Err(Refusal::Unsupported(_))));
                }
            })
            .unwrap();
        }
    }
}
