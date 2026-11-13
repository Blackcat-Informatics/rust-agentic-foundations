// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Direct attributes through Syn's native traversal, without expansion.

use super::coordinates::Coordinates;
use super::resource::{Meter, Refusal};
use syn::punctuated::Pair;
use syn::visit::Visit;

pub(super) fn expression(
    node: &syn::Expr,
    meter: &Meter,
    coordinates: &Coordinates<'_, '_>,
) -> Result<(), Refusal> {
    let mut inspect = Direct {
        meter,
        coordinates,
        failure: None,
    };
    syn::visit::visit_expr(&mut inspect, node);
    inspect.failure.map_or(Ok(()), Err)
}

pub(super) fn pattern(
    node: &syn::Pat,
    meter: &Meter,
    coordinates: &Coordinates<'_, '_>,
) -> Result<(), Refusal> {
    let mut inspect = Direct {
        meter,
        coordinates,
        failure: None,
    };
    syn::visit::visit_pat(&mut inspect, node);
    inspect.failure.map_or(Ok(()), Err)
}

pub(super) fn ty(
    node: &syn::Type,
    meter: &Meter,
    coordinates: &Coordinates<'_, '_>,
) -> Result<(), Refusal> {
    let mut inspect = Direct {
        meter,
        coordinates,
        failure: None,
    };
    syn::visit::visit_type(&mut inspect, node);
    inspect.failure.map_or(Ok(()), Err)
}

pub(super) fn list(
    attrs: &[syn::Attribute],
    meter: &Meter,
    coordinates: &Coordinates<'_, '_>,
) -> Result<(), Refusal> {
    let mut inspect = Direct {
        meter,
        coordinates,
        failure: None,
    };
    for attr in attrs {
        inspect.visit_attribute(attr);
    }
    inspect.failure.map_or(Ok(()), Err)
}

struct Direct<'m, 'source> {
    meter: &'m Meter,
    coordinates: &'m Coordinates<'source, 'm>,
    failure: Option<Refusal>,
}

impl<'ast> Visit<'ast> for Direct<'_, '_> {
    fn visit_expr(&mut self, _: &'ast syn::Expr) {}
    fn visit_pat(&mut self, _: &'ast syn::Pat) {}
    fn visit_type(&mut self, _: &'ast syn::Type) {}
    fn visit_block(&mut self, _: &'ast syn::Block) {}

    fn visit_ident(&mut self, ident: &'ast syn::Ident) {
        if self.failure.is_none() {
            self.failure = self
                .coordinates
                .range(ident.span())
                .and_then(|span| self.meter.work(span.end - span.start))
                .err();
        }
    }

    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if self.failure.is_some() {
            return;
        }
        let result = (|| {
            self.meter.work(1)?;
            for segment in attr.path().segments.pairs().map(Pair::into_value) {
                self.meter.work(1)?;
                self.visit_ident(&segment.ident);
                if let Some(cause) = self.failure {
                    return Err(cause);
                }
            }
            for inert in ["doc", "allow", "warn", "deny", "forbid", "expect"] {
                self.meter.work(inert.len())?;
                if attr.path().is_ident(inert) {
                    return Ok(());
                }
            }
            Err(Refusal::Unsupported(
                "source attributes require admitted active selection or expansion context",
            ))
        })();
        if let Err(cause) = result {
            self.failure.get_or_insert(cause);
        }
    }
}
