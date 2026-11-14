// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Location-dependent native navigation, separate from the named-view cache.

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use purrdf_rdf::viz::VizNavigation;
use raf_source::{CapturedFile, RelPath};

use super::{DiagramError, Root};
use crate::{Loaded, Workplan};

pub(crate) struct RepositoryNavigation {
    root: Rc<Root>,
    output: RelPath,
    targets: RefCell<BTreeMap<RelPath, CapturedFile>>,
}

impl RepositoryNavigation {
    pub(super) fn new(root: Rc<Root>, output: RelPath) -> Self {
        Self {
            root,
            output,
            targets: RefCell::new(BTreeMap::new()),
        }
    }

    fn destination(&self, path: &RelPath, expected: &[u8]) -> Result<VizNavigation, DiagramError> {
        let parent = path.parent().unwrap_or_else(RelPath::root);
        let directory = self.root.directory(&parent).map_err(DiagramError::data)?;
        let captured = self
            .root
            .tree()
            .capture_file(path, expected.len() as u64)
            .map_err(DiagramError::data)?;
        if captured.bytes != expected {
            return Err(DiagramError::refused(format!(
                "repository navigation destination {path} changed since loading"
            )));
        }
        directory.verify().map_err(DiagramError::data)?;
        let mut targets = self.targets.borrow_mut();
        if targets.get(path).is_some_and(|before| before != &captured) {
            return Err(DiagramError::refused(format!(
                "repository navigation destination {path} changed during generation"
            )));
        }
        targets.insert(path.clone(), captured);
        let from: Vec<_> = self
            .output
            .as_str()
            .split('/')
            .filter(|part| !part.is_empty())
            .collect();
        let to: Vec<_> = path.as_str().split('/').collect();
        let shared = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
        let relative = format!(
            "{}{}",
            "../".repeat(from.len() - shared),
            to[shared..].join("/")
        );
        Ok(VizNavigation::parse(&format!(
            "../../{}",
            purrdf_lex::percent::encode(&relative, purrdf_lex::percent::UNRESERVED_SLASH)
        ))?)
    }

    pub(crate) fn source(
        &self,
        plan: &Workplan<'_>,
        path: &RelPath,
    ) -> Result<VizNavigation, DiagramError> {
        let expected = plan.source().source(path).ok_or_else(|| {
            DiagramError::refused(format!(
                "repository navigation source {path} was not loaded"
            ))
        })?;
        self.destination(path, expected.as_bytes())
    }

    pub(crate) fn capability(
        &self,
        target: &Loaded<raf_okf::Capability>,
    ) -> Result<VizNavigation, DiagramError> {
        self.destination(
            &RelPath::new(target.path()).map_err(DiagramError::data)?,
            target.text().as_bytes(),
        )
    }

    pub(super) fn verify(&self) -> Result<(), DiagramError> {
        for expected in self.targets.borrow().values() {
            let parent = expected.path.parent().unwrap_or_else(RelPath::root);
            let _directory = self.root.directory(&parent).map_err(DiagramError::data)?;
            let actual = self
                .root
                .tree()
                .capture_file(&expected.path, expected.bytes.len() as u64)
                .map_err(DiagramError::data)?;
            if &actual != expected {
                return Err(DiagramError::refused(format!(
                    "repository navigation destination {} changed before publication",
                    expected.path
                )));
            }
        }
        Ok(())
    }
}
