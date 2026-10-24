// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One repository policy shared verbatim by the host and real check target.

use raf_census::{FileCtx, Gate, LedgerExtension, Rule, RuleFixtures, RuleHit};
use std::path::Path;

struct NoLocalCopy;

impl Rule for NoLocalCopy {
    fn id(&self) -> &'static str {
        "no-local-copy"
    }

    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        for item in &file.items {
            if let syn::Item::Fn(item) = item
                && item.sig.ident == "local_copy"
            {
                out.push(RuleHit {
                    line: item.sig.ident.span().start().line,
                    symbol: format!("{}::local_copy", ctx.package),
                    message: "use home".into(),
                });
            }
        }
    }

    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn local_copy() {}"],
            negative: vec!["fn home() {}"],
        }
    }
}

struct Owner;

impl LedgerExtension for Owner {
    fn validate(&self, _: &str, node: raf_rdf::Node<'_>) -> Option<Result<(), String>> {
        let mut values = node.values("urn:example:owner");
        let value = values.next()?;
        Some(
            if values.next().is_none()
                && value.literal() == Some(("demo", raf_rdf::vocab::STRING))
                && node
                    .properties()
                    .all(|key| key == raf_rdf::vocab::TYPE || key == "urn:example:owner")
            {
                Ok(())
            } else {
                Err("owner extension requires exactly owner demo and no unknown fields".into())
            },
        )
    }
}

pub(crate) fn registered_gate(root: impl AsRef<Path>) -> Gate {
    rule_gate(root).extensions(vec![Box::new(Owner) as Box<dyn LedgerExtension>])
}

pub(crate) fn rule_gate(root: impl AsRef<Path>) -> Gate {
    Gate::workspace(root).rules(vec![Box::new(NoLocalCopy) as Box<dyn Rule>])
}
