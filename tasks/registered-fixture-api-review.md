<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Exact registered fixture API shape

Static API inspection only; the snippets below have not been compiled. Current
host now retains bounded stderr on UTF-8/codec/mismatch errors, addressing the
initial codec-review diagnostic finding. No new codec acceptance defect found.

Use a package named demo, own [workspace], explicit edition, current-checkout
path dependencies raf-census/raf-rdf/raf-source as actually needed, and syn 3 with
full/parsing plus span locations unified through RAF's proc-macro2 dependency.
Do not use a sanitized manifest. Generate the actual lock during fixture setup,
then freeze it for the command. The outside-dependency boundary is documented
in fixture-boundary.md.

## One shared registration module

Keep one static support/registration.rs module compiled by the host integration
test and copy its exact bytes into fixture src/policy.rs. This establishes Gate
parity through the same registration implementations, without duplicating their
logic as source strings. The integration test needs syn 3 as a dev dependency
when it compiles this module. The fixture library uses mod policy and
pub use policy::registered_gate; its root owns home and optional local_copy.

```rust
use std::path::Path;
use raf_census::{FileCtx, Gate, LedgerExtension, Rule, RuleFixtures, RuleHit};

struct NoLocalCopy;
impl Rule for NoLocalCopy {
    fn id(&self) -> &'static str { "no-local-copy" }
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
    fn validate(&self, _: &str, node: raf_rdf::Node<'_>)
        -> Option<Result<(), String>>
    {
        let mut values = node.values("urn:example:owner");
        let value = values.next()?; // no claim when this owner's key is absent
        Some(if values.next().is_none()
            && value.literal() == Some(("demo", raf_rdf::vocab::STRING))
            && node.properties().all(|key| {
                key == raf_rdf::vocab::TYPE || key == "urn:example:owner"
            })
        {
            Ok(())
        } else {
            Err("owner extension requires exactly owner demo and no unknown fields".into())
        })
    }
}

pub fn registered_gate(root: impl AsRef<Path>) -> Gate {
    Gate::workspace(root)
        .rules(vec![Box::new(NoLocalCopy) as Box<dyn Rule>])
        .extensions(vec![Box::new(Owner) as Box<dyn LedgerExtension>])
}
```

RuleHit must resolve to exactly one containing census unit in that same file at
the supplied line. A root-only symbol demo or a nested module function reported
as demo::local_copy is invalid. Using top-level functions in src/lib.rs avoids
that mismatch; fixture validation separately uses package fixture. Rule IDs
must be unique lowercase kebab-case and cannot replace a built-in ID. Both rule
fixture sets are mandatory. See rule.rs and gate/rules.rs:105-129,173-187.

## Ledger and source

Construct the real helpers-ledger.ttl with raf_rdf::Builder/Triple/TermValue,
then native to_text(Format::Turtle), using existing native integer spelling.
For keys below, type means raf_rdf::vocab::TYPE; other core keys use
raf_rdf::vocab::REPORT as the Census namespace. Values are:

| Subject | Predicate | Object |
| --- | --- | --- |
| root | type | IRI REPORT + Ledger |
| root | normalForm | integer raf_census::NORMAL_FORM |
| root | reportForm | integer raf_census::REPORT_FORM |
| root | job | IRI #job |
| #job | type | IRI REPORT + LedgerJob |
| #job | jobId | string number |
| #job | summary | string Use home |
| #job | homeSymbol | string demo::home |
| #job | enforced | boolean true |
| #job | forbiddenFingerprint | string rule:no-local-copy |
| #job | extension | IRI #ext |
| #ext | type | IRI REPORT + Extension |
| #ext | urn:example:owner | string demo |

No forbiddenConstant is needed. Existing detector_ledger removes that field
before adding its rule fingerprint. Keep home documented:
`/// Home.\npub fn home()->u32{1}\n`. Clean root source additionally declares
policy/reexports registered_gate; planted source adds
`pub fn local_copy()->u32{2}\n`. Distinct bodies avoid incidental duplicate-group
findings obscuring the registered-rule assertion. The support module contains
rule snippets as strings; those are not top-level function declarations.

The extension MUST be nonempty: gate/evaluate.rs:585-588 deliberately skips
claiming a type-only extension. With owner demo it is claimed once and accepted;
owner other is claimed and refused; missing predicate leaves it unclaimed.
Registering two Owner instances must refuse duplicate ownership. Unknown core
job fields still refuse before/independently of extension ownership.

## Actual target and parity

Fixture examples/registered.rs can use this complete no-argument shape:

```rust
use std::{error::Error, io::Write as _, process::ExitCode};
fn main() -> Result<ExitCode, Box<dyn Error>> {
    let result = demo::registered_gate(".").check();
    let response = raf_census::RegisteredCheck::from_check(&result)?;
    std::io::stdout().lock().write_all(response.to_ntriples()?.as_bytes())?;
    Ok(ExitCode::from(response.exit_code()))
}
```

The host test calls registration::registered_gate(fixture.path()).check() and
compares exact findings to the shipped CLI output/status for clean and planted
neighbors. Invoke the CLI from an unrelated directory with explicit --workspace,
--package demo and --example registered. Matching clean zero/findings one/error
one response exits are mandatory; Result-returning main alone would wrongly
return zero for a successfully serialized findings response.

Also compare a bare Gate without registration: missing Rule must hard-refuse,
and Gate with Rule but no extension must refuse the nonempty extension. These
controls establish why the actual target's registrations matter. Snapshot
manifest, lock, source and debt before the asserted command and compare after;
Cargo target build artifacts are expected and are not source/debt mutation.
This concrete fixture proves real requested-root Rule/Extension transport. It
does not replace the separate pinned native provider witness or missing/build
failure controls. No reviewer compilers, tests or product edits were performed.
