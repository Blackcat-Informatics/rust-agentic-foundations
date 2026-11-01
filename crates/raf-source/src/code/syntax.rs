// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reuse the admitted target's declaration selection on the bounded parse worker.

use super::{CodeResolveError, CodeResolver};
use crate::{RelPath, SourceFile, TargetId, meter::Meter};

impl CodeResolver<'_> {
    /// Inspect an actual loaded file after its target's declared header selection.
    ///
    /// Uses the same parser and configuration filter as indexing. Spans retain
    /// their original physical bytes. Selection covers modeled declaration headers,
    /// not conditional body expressions or macro expansion. Borrowed syntax cannot
    /// escape the parse worker. Temporary selection uses the original load allowance after
    /// this query's retained reading table; it never receives a fresh allowance.
    ///
    /// # Errors
    /// Refuses absent/unconfigured targets, files not reached by that target, and
    /// original bounded parser/configuration failures with their typed causes.
    pub fn with_configured_syntax<R: Send>(
        &self,
        target: TargetId,
        path: &RelPath,
        inspect: impl FnOnce(&SourceFile, &syn::File) -> R + Send,
    ) -> Result<R, CodeResolveError> {
        let missing = |reason| CodeResolveError::SyntaxContext {
            target,
            file: path.clone(),
            reason,
        };
        let target_id = target;
        let target = self
            .workspace
            .target(target_id)
            .ok_or_else(|| missing("target is absent"))?;
        let meter = Meter::resume(self.options, self.usage);
        let refused = |error| CodeResolveError::ConfiguredSyntax(Box::new(error));
        let mut file = None;
        for candidate in self.workspace.files() {
            meter
                .work(
                    candidate
                        .path
                        .as_str()
                        .len()
                        .saturating_add(path.as_str().len())
                        .saturating_add(1),
                    path,
                    None,
                )
                .map_err(refused)?;
            if candidate.path == *path {
                file = Some(candidate);
                break;
            }
        }
        let file = file.ok_or_else(|| missing("source file is absent"))?;
        meter
            .work(file.reached.len(), path, None)
            .map_err(refused)?;
        if !file.reached.iter().any(|reach| reach.target == target_id) {
            return Err(missing("target does not reach this source file"));
        }
        let configuration = if let Some(configuration) = self.workspace.selected_configuration() {
            configuration
        } else {
            self.workspace
                .locked_binding(target.package_id)
                .ok_or_else(|| missing("supply its exact declared configuration"))?
                .configuration_for_under(target.kind, &target.name, &meter, path)
                .map_err(refused)?
                .ok_or_else(|| missing("supply its exact declared configuration"))?
        };
        let options = self.options;
        let usage = meter.usage();
        crate::parse::parse_counted_mut(path, &file.text, options.max_file_tokens, move |syntax| {
            let meter = Meter::resume(options, usage);
            crate::configuration::filter(syntax, configuration, &meter, path)?;
            Ok::<_, crate::LoadError>(inspect(file, syntax))
        })
        .map_err(|error| CodeResolveError::ConfiguredSyntax(Box::new(error)))?
        .0
        .map_err(|error| CodeResolveError::ConfiguredSyntax(Box::new(error)))
    }
}
