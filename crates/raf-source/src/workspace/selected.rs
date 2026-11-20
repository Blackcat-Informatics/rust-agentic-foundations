// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact declared target selection without dependency or provider admission.

use super::{PackageInfo, Reader, SyntaxLoad, Workspace};
use crate::configuration::{self, ActiveConfiguration, DeclaredConfiguration};
use crate::meter::Meter;
use crate::{DependencyDeclaration, LoadError, LoadOptions, RelPath, Tree};

impl Workspace {
    /// Load one exact package manifest and one declared target configuration.
    ///
    /// Workspace ancestors remain available for inherited manifest fields, but
    /// sibling packages and other targets are not indexed. Only the selected
    /// target's module tree is followed. Dependency declarations remain syntax:
    /// no dependency edge or [`Self::locked_binding`] is admitted.
    ///
    /// The configuration is supplied by the caller, not observed from Cargo or
    /// a compiler. Selection covers the existing modeled declaration headers;
    /// it does not expand macros or establish generated definitions.
    ///
    /// # Examples
    ///
    /// Declare `doc` explicitly; this does not query a compiler.
    ///
    /// ```
    /// use raf_source::{
    ///     CfgAtom, DeclaredCfg, DeclaredConfiguration, LoadOptions, MemoryTree,
    ///     RelPath, TargetKind, Workspace,
    /// };
    /// let tree = MemoryTree::from([
    ///     ("Cargo.toml", "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n"),
    ///     ("src/lib.rs", "#[cfg(doc)] pub struct Documented;"),
    /// ]);
    /// let atoms = [CfgAtom::Name("doc")];
    /// let declared = DeclaredConfiguration::new(
    ///     DeclaredCfg::complete("declared-documentation", &atoms),
    ///     &[], TargetKind::Lib, "demo",
    /// );
    /// let source = Workspace::load_configured_target(
    ///     &tree, &RelPath::new("Cargo.toml")?, declared, &LoadOptions::default(),
    /// )?;
    /// let target = source.target_ids().next().expect("one selected target");
    /// assert!(source.exported_items(target)?.items.iter()
    ///     .any(|item| item.path == "demo::Documented"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Refuses an invalid or missing package manifest, absent/ambiguous target,
    /// invalid inherited fields, incomplete required features or configuration,
    /// and all ordinary contained source and resource failures.
    pub fn load_configured_target(
        tree: &dyn Tree,
        manifest: &RelPath,
        configuration: DeclaredConfiguration<'_>,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        Self::load_selected(
            tree,
            manifest,
            options,
            SyntaxLoad::Configured(configuration),
        )
    }

    /// Load one declared Cargo target using the bounded native Linux parser workers.
    ///
    /// The manifest, target and declaration selection match
    /// [`Self::load_configured_target`]. Only parsing is speculative. The
    /// canonical coordinator admits selection and facts using the original
    /// source and configuration budgets. The tree must satisfy
    /// [`crate::ParallelTree`]'s stable observation contract.
    ///
    /// # Errors
    /// Returns the canonical input errors of [`Self::load_configured_target`],
    /// or scheduler-storage and worker-start failures. All workers are joined.
    ///
    /// # Panics
    /// Resumes a canonically reached parser panic after worker cleanup.
    #[cfg(target_os = "linux")]
    pub fn par_load_configured_target(
        tree: &dyn crate::ParallelTree,
        manifest: &RelPath,
        configuration: DeclaredConfiguration<'_>,
        options: &LoadOptions,
        parallel: &crate::ParallelOptions,
    ) -> Result<Self, LoadError> {
        Self::load_selected(
            tree,
            manifest,
            options,
            SyntaxLoad::ConfiguredParallel(configuration, parallel),
        )
    }

    fn load_selected(
        tree: &dyn Tree,
        manifest: &RelPath,
        options: &LoadOptions,
        syntax: SyntaxLoad<'_>,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        meter.items(crate::index::LOAD_BASE_BYTES, manifest, None)?;
        let discovered = crate::discovery::single(tree, manifest, &meter)?;
        Self::build_discovered_with_syntax(tree, options, &meter, None, &discovered, syntax)
    }
}

pub(super) fn configure(
    reader: &Reader<'_>,
    declarations: &[Vec<DependencyDeclaration>],
    infos: &mut [PackageInfo],
    targets: &mut Vec<(&RelPath, crate::target::Derived)>,
    input: DeclaredConfiguration<'_>,
) -> Result<ActiveConfiguration, LoadError> {
    let [found] = reader.found else {
        return Err(configuration::failure(
            &RelPath::root(),
            "select exactly one package manifest",
        ));
    };
    let manifest = &found.package.manifest;
    let active = configuration::admit(
        input,
        &found.manifest.features,
        &declarations[0],
        reader.meter,
        manifest,
    )?;
    let mut selected = None;
    for (index, (_, derived)) in targets.iter().enumerate() {
        reader.meter.work(
            derived
                .target
                .name
                .len()
                .saturating_add(active.target_name().len())
                .saturating_add(1),
            manifest,
            None,
        )?;
        if derived.target.kind == active.target_kind()
            && derived.target.name == active.target_name()
        {
            if selected.replace(index).is_some() {
                return Err(configuration::failure(
                    manifest,
                    "declared kind/name selects multiple package targets",
                ));
            }
            for feature in &derived.target.required_features {
                if !configuration::contains(active.features(), feature, reader.meter, manifest)? {
                    return Err(configuration::failure(
                        manifest,
                        "selected target requires an inactive feature",
                    ));
                }
            }
        }
    }
    let selected = selected.ok_or_else(|| {
        configuration::failure(manifest, "declared kind/name selects no package target")
    })?;
    let target = targets.remove(selected);
    targets.clear();
    targets.push(target);
    infos[0].configured = true;
    for dependency in &mut infos[0].dependencies {
        dependency.package = None;
    }
    Ok(active)
}
