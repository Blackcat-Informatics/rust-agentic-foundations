// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked virtual preparation available to supplied scripted runner implementations.

use std::sync::{Arc, Mutex};

use super::{PreparedView, ViewContext, ViewError, ViewRequest};
use crate::exec::ExecError;

/// Runner-owned virtual contexts over complete checked native captures.
///
/// A custom scripted runner delegates preparation and context validation here,
/// then explicitly answers its prepared invocations. This never claims native
/// process execution or a physical materialization. Contexts cannot be shared
/// with another service instance or used after release.
#[derive(Debug, Default)]
pub struct VirtualViews {
    owner: Arc<u8>,
    next: Mutex<u64>,
    active: Mutex<Vec<ViewContext>>,
}
impl VirtualViews {
    /// An authority with no accepted virtual views.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture the request's complete source and admit a new virtual context.
    /// # Errors
    /// Native capture failures, limits and poisoned ownership state refuse.
    pub fn prepare(&self, request: &ViewRequest) -> Result<PreparedView, ExecError> {
        let tree = raf_source::DiskTree::new(request.root())
            .map_err(|e| ViewError::new("open virtual source", e))?;
        let administration = raf_source::RelPath::new(".git")
            .map_err(|e| ViewError::new("virtual administration path", e))?;
        if raf_source::Tree::entry(&tree, &administration)
            .map_err(|e| ViewError::new("inspect virtual source", e))?
            .is_some()
        {
            return Err(ViewError::refusal("prepare virtual source", "Git administration requires native SystemRunner preparation; virtual fixtures must supply their tracked oracle explicitly").into());
        }
        let slot = {
            let mut next = self
                .next
                .lock()
                .map_err(|e| ViewError::refusal("allocate virtual context", e.to_string()))?;
            let slot = *next;
            *next = next.checked_add(1).ok_or_else(|| {
                ViewError::refusal("prepare virtual view", "context space exhausted")
            })?;
            slot
        };
        let view = PreparedView::virtual_view(request, self.owner.clone(), slot)?;
        self.active
            .lock()
            .map_err(|e| ViewError::refusal("prepare virtual view", e.to_string()))?
            .push(view.context().clone());
        Ok(view)
    }

    /// Require an active context owned by this exact virtual authority.
    /// # Errors
    /// Foreign, unknown and released contexts refuse before scripted execution.
    pub fn validate(&self, context: &ViewContext) -> Result<(), ExecError> {
        if self
            .active
            .lock()
            .map_err(|e| ViewError::refusal("validate virtual context", e.to_string()))?
            .contains(context)
        {
            Ok(())
        } else {
            Err(ViewError::refusal(
                "validate virtual context",
                "unknown, foreign or released prepared context",
            )
            .into())
        }
    }

    /// Compare the original source with its accepted immutable virtual model.
    /// # Errors
    /// Source changes and native capture failures refuse publication.
    pub fn revalidate(&self, view: &PreparedView) -> Result<(), ExecError> {
        self.validate(view.context())?;
        let actual = PreparedView::virtual_view(
            view.captured().request(),
            view.context.authority.clone(),
            view.context.slot,
        )?;
        if actual.captured().capture == view.captured().capture {
            Ok(())
        } else {
            Err(view
                .captured()
                .capture
                .changed(&actual.captured().capture, "revalidate virtual source")
                .into())
        }
    }

    /// Release the virtual context; there are no physical resources to remove.
    /// # Errors
    /// Foreign, unknown or already released contexts refuse explicitly.
    pub fn finish(&self, view: &PreparedView) -> Result<(), ExecError> {
        let mut active = self
            .active
            .lock()
            .map_err(|e| ViewError::refusal("release virtual view", e.to_string()))?;
        let index = active
            .iter()
            .position(|context| context == view.context())
            .ok_or_else(|| {
                ViewError::refusal(
                    "release virtual view",
                    "unknown, foreign or released prepared context",
                )
            })?;
        active.remove(index);
        Ok(())
    }
}
