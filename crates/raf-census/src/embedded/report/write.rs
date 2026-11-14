// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual source provenance and stable candidate support in admitted native RDF.

use super::{FORM, OWNER_NUMBERS, OWNER_TEXT, PRELUDE_PROFILE, VOCAB};
use crate::ReportError as E;
use crate::embedded::{
    analysis::{Function, Origin},
    names,
    output::{BodyFailure, Failure, OutputFailure},
    pairing::{Candidate, Matches, Site},
    policy,
    resource::{Refusal, Scratch, Text},
    schema::{Emitter, Graph, Named, Schema},
};
use raf_rdf::{AdmittedGraph, ConstructionExit, ConstructionFailure};

struct Writer<'b, 's, 'm, 'run> {
    emit: Emitter<'b, 's, 'm, 'run>,
    schema: Schema<'run>,
    sources: Named<'run, ()>,
    regions: Named<'run, Site>,
}

impl<'run> Writer<'_, '_, '_, 'run> {
    fn source(&mut self, file: &raf_source::SourceFile) -> Result<Text<'run>, BodyFailure<'run>> {
        let authority = Text::display(self.schema.meter, &file.path.authority().index())?;
        let id = self
            .schema
            .key("source", [authority.as_ref(), file.path.as_str()])?;
        if !self.sources.remember(&id, ())? {
            return Ok(id);
        }
        self.emit.class(&id, "Source")?;
        self.emit
            .number(&id, "sourceAuthority", file.path.authority().index())?;
        self.emit.text(&id, "file", file.path.as_str())?;
        let mut hash = purrdf_hash::blake3::Hasher::new();
        let mut lines = 1usize;
        for extent in file.text.as_bytes().chunks(256) {
            self.schema.meter.work(extent.len())?;
            hash.update(extent);
            self.schema.meter.work(extent.len())?;
            let mut at = 0;
            while let Some(newline) = purrdf_lex::scan::find_byte(&extent[at..], b'\n') {
                self.schema.meter.work(1)?;
                lines = lines.checked_add(1).ok_or(Failure::Overflow)?;
                at += newline + 1;
            }
        }
        self.schema.meter.work(1)?;
        self.emit.display(
            &id,
            "sourceDigest",
            &purrdf_hash::hex::Lower(hash.finalize().as_bytes()),
            raf_rdf::vocab::STRING,
        )?;
        self.emit.number(&id, "sourceBytes", file.text.len())?;
        self.emit.number(&id, "sourceLines", lines)?;
        Ok(id)
    }
    fn owner(
        &mut self,
        function: &str,
        workspace: &raf_source::Workspace,
        item: &raf_source::Item,
    ) -> Result<Text<'run>, BodyFailure<'run>> {
        let crate::embedded::identity::Owner {
            item,
            target,
            package,
        } = crate::embedded::identity::owner(workspace, item)
            .map_err(|reason| self.schema.error(E::Inconsistent, &reason))?;
        let kind = crate::embedded::identity::target_kind(target.kind)
            .map_err(|reason| self.schema.error(E::Value, &reason))?;
        let texts = [
            package.manifest.as_str(),
            &package.name,
            target.root.as_str(),
            kind,
            &target.name,
            &target.crate_path,
            item.path.as_str(),
        ];
        let numbers = [
            package.manifest.authority().index(),
            target.root.authority().index(),
            item.span.start_byte,
            item.span.end_byte,
        ];
        let mut numeric = Scratch::new(self.schema.meter);
        for value in numbers {
            numeric.push(Text::display(self.schema.meter, &value)?)?;
        }
        let id = self.schema.key(
            "owner",
            std::iter::once(function)
                .chain(texts)
                .chain(numeric.iter().map(Text::as_ref)),
        )?;
        self.emit.class(&id, "Owner")?;
        self.emit.iri(&id, "function", function)?;
        for (field, value) in OWNER_TEXT.iter().zip(texts) {
            self.emit.text(&id, field, value)?;
        }
        for (field, value) in OWNER_NUMBERS.iter().zip(numbers) {
            self.emit.number(&id, field, value)?;
        }
        Ok(id)
    }
    fn function(&mut self, function: &Function<'_>) -> Result<(), BodyFailure<'run>> {
        let (workspace, file, owners, profile) = match &function.origin {
            Origin::Source {
                workspace,
                file,
                owners,
                profile,
            } => (*workspace, *file, owners, *profile),
            #[cfg(test)]
            Origin::Text => {
                return Err(self
                    .schema
                    .error(E::Value, &"raw text is not report source authority")
                    .into());
            }
        };
        if !workspace
            .file(&file.path)
            .is_some_and(|actual| std::ptr::eq(actual, file))
            || !std::ptr::eq(function.source, file.text.as_str())
            || owners.is_empty()
        {
            return Err(self
                .schema
                .error(
                    E::Inconsistent,
                    &"function lost actual workspace source provenance",
                )
                .into());
        }
        let id = function_id(self.schema, function)?;
        let source = self.source(file)?;
        let (disabled, no_std) = profile.flags();
        self.emit.class(&id, "Function")?;
        self.emit.iri(&id, "source", &source)?;
        for (field, value) in [
            ("startByte", function.facts.body.start),
            ("endByte", function.facts.body.end),
            ("atomCount", function.tokens.atoms.len()),
            ("edition", usize::from(profile.edition())),
        ] {
            self.emit.number(&id, field, value)?;
        }
        self.emit
            .display(&id, "noImplicitPrelude", &disabled, raf_rdf::vocab::BOOLEAN)?;
        self.emit
            .display(&id, "noStd", &no_std, raf_rdf::vocab::BOOLEAN)?;
        for item in owners {
            self.schema.meter.work(1)?;
            if item.file != file.path
                || item.body.as_ref().is_none_or(|body| {
                    body.start_byte != function.facts.body.start
                        || body.end_byte != function.facts.body.end
                })
            {
                return Err(self
                    .schema
                    .error(E::Inconsistent, &"owner body differs from actual function")
                    .into());
            }
            let owner = self.owner(&id, workspace, item)?;
            self.emit.iri(&id, "owner", &owner)?;
        }
        self.emit.iri("", "function", &id)
    }
    fn candidate(&mut self, seed: &Seed<'run>) -> Result<(), BodyFailure<'run>> {
        self.emit.class(&seed.id, "Candidate")?;
        self.emit.iri(&seed.id, "leftFunction", &seed.left)?;
        self.emit.iri(&seed.id, "rightFunction", &seed.right)?;
        self.emit.number(&seed.id, "leftAtom", seed.a)?;
        self.emit.number(&seed.id, "rightAtom", seed.b)?;
        self.emit.text(&seed.id, "seedDigest", &seed.digest)?;
        if let Some((a, b, len)) = seed.alignment {
            self.emit.number(&seed.id, "alignmentLeft", a)?;
            self.emit.number(&seed.id, "alignmentRight", b)?;
            self.emit.number(&seed.id, "alignmentLength", len)?;
        }
        self.emit.iri("", "candidate", &seed.id)
    }
    fn region(&mut self, region: &Region<'run>) -> Result<(), BodyFailure<'run>> {
        let site = region.site;
        if let Some(prior) = self.regions.get(&region.id)? {
            self.schema.meter.work(6)?;
            if (
                prior.bytes,
                prior.first_line,
                prior.last_line,
                prior.atom_start,
                prior.atom_end,
            ) != (
                site.bytes,
                site.first_line,
                site.last_line,
                site.atom_start,
                site.atom_end,
            ) {
                return Err(self
                    .schema
                    .error(E::Inconsistent, &"shared region has different source facts")
                    .into());
            }
            return Ok(());
        }
        self.regions.remember(&region.id, site)?;
        self.emit.class(&region.id, "Region")?;
        self.emit.iri(&region.id, "function", &region.function)?;
        for (field, value) in [
            ("startByte", site.bytes.start),
            ("endByte", site.bytes.end),
            ("firstLine", site.first_line),
            ("lastLine", site.last_line),
            ("atomStart", site.atom_start),
            ("atomEnd", site.atom_end),
        ] {
            self.emit.number(&region.id, field, value)?;
        }
        Ok(())
    }
}

pub(super) fn function_id<'m>(
    schema: Schema<'m>,
    function: &Function<'_>,
) -> Result<Text<'m>, Failure<'m>> {
    let (file, _) = function
        .origin
        .source()
        .ok_or_else(|| schema.error(E::Value, &"actual source provenance required"))?;
    let authority = Text::display(schema.meter, &file.path.authority().index())?;
    let start = Text::display(schema.meter, &function.facts.body.start)?;
    let end = Text::display(schema.meter, &function.facts.body.end)?;
    schema.key(
        "function",
        [
            authority.as_ref(),
            file.path.as_str(),
            start.as_ref(),
            end.as_ref(),
        ],
    )
}

fn input<'a, 'source, 'm>(
    schema: Schema<'m>,
    matches: &'a Matches<'source>,
    index: usize,
) -> Result<&'a Function<'source>, Failure<'m>> {
    matches
        .analysis
        .functions
        .get(index)
        .ok_or_else(|| schema.error(E::Inconsistent, &"missing source function"))
}

struct Seed<'m> {
    id: Text<'m>,
    left: Text<'m>,
    right: Text<'m>,
    a: usize,
    b: usize,
    digest: Text<'m>,
    alignment: Option<(usize, usize, usize)>,
}
fn seed<'m>(
    schema: Schema<'m>,
    matches: &Matches<'_>,
    candidate: Candidate,
) -> Result<Seed<'m>, Failure<'m>> {
    let mut left = function_id(schema, input(schema, matches, candidate.left.function)?)?;
    let mut right = function_id(schema, input(schema, matches, candidate.right.function)?)?;
    let (mut a, mut b) = (candidate.left.atom, candidate.right.atom);
    let mut alignment = candidate
        .alignment
        .map(|alignment| (alignment.left.atom, alignment.right.atom, alignment.len));
    let order = schema.meter.compare(&right, &left)?;
    if order.is_lt() || order.is_eq() && b < a {
        std::mem::swap(&mut left, &mut right);
        std::mem::swap(&mut a, &mut b);
        if let Some((a, b, _)) = &mut alignment {
            std::mem::swap(a, b);
        }
    }
    let digest = Text::display(
        schema.meter,
        &purrdf_hash::hex::Lower(&candidate.seed_digest),
    )?;
    let x = Text::display(schema.meter, &a)?;
    let y = Text::display(schema.meter, &b)?;
    let id = schema.key(
        "candidate",
        [
            left.as_ref(),
            right.as_ref(),
            x.as_ref(),
            y.as_ref(),
            digest.as_ref(),
        ],
    )?;
    Ok(Seed {
        id,
        left,
        right,
        a,
        b,
        digest,
        alignment,
    })
}

struct Region<'m> {
    id: Text<'m>,
    function: Text<'m>,
    site: Site,
}
fn region<'m>(
    schema: Schema<'m>,
    matches: &Matches<'_>,
    site: Site,
) -> Result<Region<'m>, Failure<'m>> {
    let function = function_id(schema, input(schema, matches, site.function)?)?;
    let mut numbers = Scratch::new(schema.meter);
    for value in [
        site.bytes.start,
        site.bytes.end,
        site.atom_start,
        site.atom_end,
    ] {
        numbers.push(Text::display(schema.meter, &value)?)?;
    }
    let id = schema.key(
        "region",
        std::iter::once(function.as_ref()).chain(numbers.iter().map(Text::as_ref)),
    )?;
    Ok(Region { id, function, site })
}
fn supports(
    schema: Schema<'_>,
    seed: &Seed<'_>,
    left: &Region<'_>,
    right: &Region<'_>,
) -> Result<bool, Refusal> {
    let Some((a, b, len)) = seed.alignment else {
        return Ok(false);
    };
    Ok(schema.meter.compare(&seed.left, &left.function)?.is_eq()
        && schema.meter.compare(&seed.right, &right.function)?.is_eq()
        && left.site.atom_start.checked_sub(a) == right.site.atom_start.checked_sub(b)
        && left.site.atom_start >= a
        && right.site.atom_start >= b
        && a.checked_add(len)
            .is_some_and(|end| left.site.atom_end <= end)
        && b.checked_add(len)
            .is_some_and(|end| right.site.atom_end <= end))
}

pub(super) fn graph<'m, R>(
    matches: &'m Matches<'_>,
    observe: impl for<'f> FnOnce(Result<Graph<'m>, OutputFailure<'f>>) -> R,
) -> R {
    let meter = matches.analysis.meter();
    let schema = Schema {
        meter,
        vocab: VOCAB,
        label: "embedded",
    };
    AdmittedGraph::build_observed(
        meter.native(),
        meter.capture(),
        |build| {
            let result = (|| {
                let mut writer = Writer {
                    emit: Emitter { build, schema },
                    schema,
                    sources: Named::new(meter),
                    regions: Named::new(meter),
                };
                writer.emit.class("", "EmbeddedReport")?;
                writer.emit.number("", "reportForm", FORM)?;
                for (field, value) in [
                    ("matchingProfile", policy::PROFILE),
                    ("identifierProfile", names::PROFILE),
                    ("preludeProfile", PRELUDE_PROFILE),
                ] {
                    writer.emit.text("", field, value)?;
                }
                for function in &matches.analysis.functions {
                    meter.work(1)?;
                    writer.function(function)?;
                }
                let mut seeds = Scratch::new(meter);
                for value in &matches.candidates {
                    meter.work(1)?;
                    let seed = seed(schema, matches, *value)?;
                    writer.candidate(&seed)?;
                    seeds.push(seed)?;
                }
                for witness in &matches.witnesses {
                    meter.work(1)?;
                    let mut left = region(schema, matches, witness.left)?;
                    let mut right = region(schema, matches, witness.right)?;
                    let order = meter.compare(&right.function, &left.function)?;
                    if order.is_lt()
                        || order.is_eq()
                            && (right.site.atom_start, right.site.atom_end)
                                < (left.site.atom_start, left.site.atom_end)
                    {
                        std::mem::swap(&mut left, &mut right);
                    }
                    let mut support: Option<&str> = None;
                    for candidate in seeds.iter() {
                        meter.work(1)?;
                        if supports(schema, candidate, &left, &right)? {
                            let earlier = match support {
                                None => true,
                                Some(prior) => meter.compare(&candidate.id, prior)?.is_lt(),
                            };
                            if earlier {
                                support = Some(&candidate.id);
                            }
                        }
                    }
                    let support = support.ok_or_else(|| {
                        schema.error(
                            E::Inconsistent,
                            &"witness has no supporting coarse candidate",
                        )
                    })?;
                    writer.region(&left)?;
                    writer.region(&right)?;
                    let digest = Text::display(meter, &purrdf_hash::hex::Lower(&witness.digest))?;
                    let id = schema.key(
                        "witness",
                        [left.id.as_ref(), right.id.as_ref(), digest.as_ref()],
                    )?;
                    writer.emit.class(&id, "Witness")?;
                    writer.emit.iri(&id, "left", &left.id)?;
                    writer.emit.iri(&id, "right", &right.id)?;
                    writer.emit.text(&id, "digest", &digest)?;
                    writer.emit.iri(&id, "candidate", support)?;
                    writer.emit.iri("", "witness", &id)?;
                }
                Ok::<(), BodyFailure<'m>>(())
            })();
            result.map_err(|cause| match cause {
                BodyFailure::Native(stopped) => ConstructionExit::Stopped(stopped),
                BodyFailure::Caller(cause) => ConstructionExit::Caller(cause),
            })
        },
        |result| match result {
            Ok(graph) => observe(Ok(graph)),
            Err(ConstructionFailure::Native(cause)) => observe(Err(OutputFailure::build(&cause))),
            Err(ConstructionFailure::Caller(cause)) => {
                observe(Err(OutputFailure::failure(cause.cause())))
            }
        },
    )
}
