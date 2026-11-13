// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed admitted shapes and relationships of inert evidence claims.

use super::{FORM, OWNER_NUMBERS, OWNER_TEXT, PRELUDE_PROFILE, VOCAB};
use crate::ReportError as E;
use crate::embedded::{
    discovery::GRAM,
    names,
    output::Failure,
    policy,
    resource::{Meter, Text},
    schema::{BorrowedSet, Graph, Node, Schema},
};

type Rule = (&'static str, usize, usize);
const MANY: usize = usize::MAX;
struct Reader<'a, 'm> {
    graph: &'a Graph<'m>,
    schema: Schema<'m>,
}

impl<'a, 'm> Reader<'a, 'm> {
    fn error(&self, variant: fn(String) -> E, value: &impl std::fmt::Display) -> Failure<'m> {
        self.schema.error(variant, value)
    }
    fn eq(&self, left: &str, right: &str) -> Result<bool, Failure<'m>> {
        Ok(self.schema.meter.compare(left, right)?.is_eq())
    }
    fn rules(&self, class: &str) -> Result<impl Iterator<Item = Rule> + Clone, Failure<'m>> {
        self.schema.meter.work(class.len())?;
        let required: &'static [&'static str] = match class {
            "EmbeddedReport" => &[
                "reportForm",
                "matchingProfile",
                "identifierProfile",
                "preludeProfile",
            ],
            "Source" => &[
                "sourceAuthority",
                "file",
                "sourceDigest",
                "sourceBytes",
                "sourceLines",
            ],
            "Function" => &[
                "source",
                "startByte",
                "endByte",
                "atomCount",
                "edition",
                "noImplicitPrelude",
                "noStd",
            ],
            "Owner" => &["function"],
            "Region" => &[
                "function",
                "startByte",
                "endByte",
                "firstLine",
                "lastLine",
                "atomStart",
                "atomEnd",
            ],
            "Candidate" => &[
                "leftFunction",
                "rightFunction",
                "leftAtom",
                "rightAtom",
                "seedDigest",
            ],
            "Witness" => &["left", "right", "digest", "candidate"],
            _ => return Err(self.error(E::Class, &class)),
        };
        let extra: &'static [Rule] = match class {
            "EmbeddedReport" => &[
                ("function", 0, MANY),
                ("candidate", 0, MANY),
                ("witness", 0, MANY),
            ],
            "Function" => &[("owner", 1, MANY)],
            "Candidate" => &[
                ("alignmentLeft", 0, 1),
                ("alignmentRight", 0, 1),
                ("alignmentLength", 0, 1),
            ],
            _ => &[],
        };
        let (owner_text, owner_numbers) = if class == "Owner" {
            (OWNER_TEXT, OWNER_NUMBERS)
        } else {
            (&[][..], &[][..])
        };
        Ok(std::iter::once(("type", 1, 1))
            .chain(
                required
                    .iter()
                    .chain(owner_text)
                    .chain(owner_numbers)
                    .map(|key| (*key, 1, 1)),
            )
            .chain(extra.iter().copied()))
    }
    fn shape(&self, node: Node<'a>) -> Result<(), Failure<'m>> {
        let fields = self.rules(self.schema.class(node)?)?;
        for &(_, predicate) in self.schema.properties(node)?.iter() {
            let mut known = false;
            for (field, _, _) in fields.clone() {
                let property = self.schema.property(field)?;
                if self.eq(&property, predicate)? {
                    known = true;
                    break;
                }
            }
            if !known {
                return Err(self.error(E::UnknownPredicate, &predicate));
            }
        }
        for (field, min, max) in fields {
            let amount = self.schema.amount(node, field)?;
            if amount < min {
                return Err(self.error(E::Missing, &format_args!("embedded:{field}")));
            }
            if amount > max {
                return Err(self.error(E::Repeated, &format_args!("embedded:{field}")));
            }
        }
        Ok(())
    }
    fn typed(&self, id: &str, expected: &str) -> Result<Node<'a>, Failure<'m>> {
        let node = self.schema.get(self.graph, id)?;
        if !self.eq(self.schema.class(node)?, expected)? {
            return Err(self.error(E::Inconsistent, &format_args!("{id}: {expected} required")));
        }
        Ok(node)
    }
    fn target(&self, node: Node<'a>, field: &str, expected: &str) -> Result<Node<'a>, Failure<'m>> {
        self.typed(self.schema.reference(node, field)?, expected)
    }
    fn root(&self, node: Node<'a>) -> Result<(), Failure<'m>> {
        if self.schema.count(node, "reportForm")? != FORM
            || !self.eq(self.schema.text(node, "matchingProfile")?, policy::PROFILE)?
            || !self.eq(self.schema.text(node, "identifierProfile")?, names::PROFILE)?
            || !self.eq(self.schema.text(node, "preludeProfile")?, PRELUDE_PROFILE)?
        {
            return Err(self.error(
                E::Value,
                &"unsupported embedded report or declared language profile",
            ));
        }
        Ok(())
    }
    fn path(&self, node: Node<'a>, field: &str, manifest: bool) -> Result<(), Failure<'m>> {
        let original = self.schema.text(node, field)?;
        self.schema.meter.work(original.len())?;
        let file_name = original.rsplit('/').next().filter(|name| !name.is_empty());
        self.schema.meter.work(original.len())?;
        if !raf_source::RelPath::is_canonical(original)
            || file_name.is_none()
            || manifest && file_name != Some("Cargo.toml")
        {
            return Err(self.error(
                E::Value,
                &format_args!("embedded:{field}: source-relative file required"),
            ));
        }
        Ok(())
    }
    fn function_key(
        &self,
        authority: usize,
        path: &str,
        start: usize,
        end: usize,
    ) -> Result<Text<'m>, Failure<'m>> {
        let authority = Text::display(self.schema.meter, &authority)?;
        self.schema
            .numbered_key("function", &[authority.as_ref(), path], &[start, end], &[])
    }
    fn source(&self, id: &str, node: Node<'a>) -> Result<(), Failure<'m>> {
        self.path(node, "file", false)?;
        let expected = self.schema.numbered_key(
            "source",
            &[],
            &[self.schema.count(node, "sourceAuthority")?],
            &[self.schema.text(node, "file")?],
        )?;
        if !self.eq(id, &expected)? {
            return Err(self.error(E::Name, &id));
        }
        self.schema.digest(node, "sourceDigest")?;
        let bytes = self.schema.count(node, "sourceBytes")?;
        let lines = self.schema.count(node, "sourceLines")?;
        if lines == 0 || lines - 1 > bytes {
            return Err(self.error(E::Inconsistent, &"source line bound"));
        }
        Ok(())
    }
    fn function(&self, id: &str, node: Node<'a>) -> Result<(), Failure<'m>> {
        let source = self.target(node, "source", "Source")?;
        let (start, end) = (
            self.schema.count(node, "startByte")?,
            self.schema.count(node, "endByte")?,
        );
        if start >= end
            || end > self.schema.count(source, "sourceBytes")?
            || self.schema.count(node, "atomCount")? < 2
        {
            return Err(self.error(E::Inconsistent, &"function range or original atom count"));
        }
        let expected = self.function_key(
            self.schema.count(source, "sourceAuthority")?,
            self.schema.text(source, "file")?,
            start,
            end,
        )?;
        if !self.eq(id, &expected)? {
            return Err(self.error(E::Name, &id));
        }
        if !matches!(
            self.schema.count(node, "edition")?,
            2015 | 2018 | 2021 | 2024
        ) {
            return Err(self.error(E::Value, &"unsupported declared embedded edition"));
        }
        self.schema.boolean(node, "noImplicitPrelude")?;
        self.schema.boolean(node, "noStd")?;
        for &value in self.schema.values(node, "owner")?.iter() {
            let owner = self.typed(
                self.schema
                    .iri(value)?
                    .ok_or_else(|| self.error(E::Value, &"owner reference required"))?,
                "Owner",
            )?;
            if !self.eq(self.schema.reference(owner, "function")?, id)? {
                return Err(self.error(E::Inconsistent, &"owner belongs to a different function"));
            }
        }
        Ok(())
    }
    fn owner(&self, id: &str, node: Node<'a>) -> Result<(), Failure<'m>> {
        let function_id = self.schema.reference(node, "function")?;
        let function = self.typed(function_id, "Function")?;
        let source = self.target(function, "source", "Source")?;
        let mut parts = crate::embedded::resource::Scratch::new(self.schema.meter);
        parts.push(function_id)?;
        for field in OWNER_TEXT {
            parts.push(self.schema.text(node, field)?)?;
        }
        let mut numbers = crate::embedded::resource::Scratch::new(self.schema.meter);
        for field in OWNER_NUMBERS {
            numbers.push(Text::display(
                self.schema.meter,
                &self.schema.count(node, field)?,
            )?)?;
        }
        let expected = self.schema.key(
            "owner",
            parts
                .iter()
                .copied()
                .chain(numbers.iter().map(Text::as_ref)),
        )?;
        if !self.eq(id, &expected)? {
            return Err(self.error(E::Name, &id));
        }
        self.path(node, "manifest", true)?;
        self.path(node, "targetRoot", false)?;
        for field in OWNER_TEXT {
            if self.schema.text(node, field)?.is_empty() {
                return Err(self.error(E::Value, &format_args!("empty owner {field}")));
            }
        }
        let kind = self.schema.text(node, "targetKind")?;
        self.schema.meter.work(kind.len())?;
        if !matches!(
            kind,
            "lib" | "bin" | "test" | "bench" | "example" | "build-script"
        ) {
            return Err(self.error(E::Value, &"unsupported owner target kind"));
        }
        if self.schema.count(node, "itemStartByte")? > self.schema.count(function, "startByte")?
            || self.schema.count(node, "itemEndByte")? < self.schema.count(function, "endByte")?
            || self.schema.count(node, "itemEndByte")? > self.schema.count(source, "sourceBytes")?
        {
            return Err(self.error(E::Inconsistent, &"owner does not contain its body"));
        }
        let mut referenced = false;
        for &value in self.schema.values(function, "owner")?.iter() {
            if let Some(iri) = self.schema.iri(value)?
                && self.eq(iri, id)?
            {
                referenced = true;
                break;
            }
        }
        if !referenced {
            return Err(self.error(E::Orphan, &id));
        }
        Ok(())
    }
    fn region(&self, id: &str, node: Node<'a>) -> Result<(), Failure<'m>> {
        let function_id = self.schema.reference(node, "function")?;
        let function = self.typed(function_id, "Function")?;
        let source = self.target(function, "source", "Source")?;
        let (start, end) = (
            self.schema.count(node, "startByte")?,
            self.schema.count(node, "endByte")?,
        );
        let (first, last) = (
            self.schema.count(node, "firstLine")?,
            self.schema.count(node, "lastLine")?,
        );
        let (a, b) = (
            self.schema.count(node, "atomStart")?,
            self.schema.count(node, "atomEnd")?,
        );
        if start >= end
            || start < self.schema.count(function, "startByte")?
            || end > self.schema.count(function, "endByte")?
            || a >= b
            || b > self.schema.count(function, "atomCount")?
            || first == 0
            || first > last
            || last > self.schema.count(source, "sourceLines")?
        {
            return Err(self.error(E::Inconsistent, &"region source, line or atom range"));
        }
        let expected =
            self.schema
                .numbered_key("region", &[function_id], &[start, end, a, b], &[])?;
        if !self.eq(id, &expected)? {
            return Err(self.error(E::Name, &id));
        }
        Ok(())
    }
    fn alignment(&self, node: Node<'a>) -> Result<Option<(usize, usize, usize)>, Failure<'m>> {
        let fields = ["alignmentLeft", "alignmentRight", "alignmentLength"];
        let mut present = [false; 3];
        for (slot, field) in present.iter_mut().zip(fields) {
            *slot = self.schema.exists(node, field)?;
        }
        match present {
            [false, false, false] => Ok(None),
            [true, true, true] => Ok(Some((
                self.schema.count(node, fields[0])?,
                self.schema.count(node, fields[1])?,
                self.schema.count(node, fields[2])?,
            ))),
            _ => Err(self.error(E::Inconsistent, &"partial coarse alignment")),
        }
    }
    fn candidate(&self, id: &str, node: Node<'a>) -> Result<(), Failure<'m>> {
        let left = self.schema.reference(node, "leftFunction")?;
        let right = self.schema.reference(node, "rightFunction")?;
        let l = self.typed(left, "Function")?;
        let r = self.typed(right, "Function")?;
        let (a, b) = (
            self.schema.count(node, "leftAtom")?,
            self.schema.count(node, "rightAtom")?,
        );
        let (left_count, right_count) = (
            self.schema.count(l, "atomCount")?,
            self.schema.count(r, "atomCount")?,
        );
        let order = self.schema.meter.compare(left, right)?;
        if order.is_gt()
            || order.is_eq() && a >= b
            || !fits(a, GRAM, left_count)
            || !fits(b, GRAM, right_count)
        {
            return Err(self.error(E::Inconsistent, &"candidate seed identity or atom range"));
        }
        let expected = self.schema.numbered_key(
            "candidate",
            &[left, right],
            &[a, b],
            &[self.schema.digest(node, "seedDigest")?],
        )?;
        if !self.eq(id, &expected)? {
            return Err(self.error(E::Name, &id));
        }
        if let Some((x, y, len)) = self.alignment(node)?
            && (len < GRAM
                || x > a
                || y > b
                || !fits(x, len, left_count)
                || !fits(y, len, right_count)
                || !fits(a - x, GRAM, len)
                || !fits(b - y, GRAM, len)
                || a - x != b - y)
        {
            return Err(self.error(
                E::Inconsistent,
                &"coarse alignment does not contain its seed",
            ));
        }
        Ok(())
    }
    fn supports(
        &self,
        candidate: Node<'a>,
        left: Node<'a>,
        right: Node<'a>,
    ) -> Result<bool, Failure<'m>> {
        let Some((x, y, len)) = self.alignment(candidate)? else {
            return Ok(false);
        };
        let (a, b) = (
            self.schema.count(left, "atomStart")?,
            self.schema.count(right, "atomStart")?,
        );
        Ok(self.eq(
            self.schema.reference(candidate, "leftFunction")?,
            self.schema.reference(left, "function")?,
        )? && self.eq(
            self.schema.reference(candidate, "rightFunction")?,
            self.schema.reference(right, "function")?,
        )? && a >= x
            && b >= y
            && a - x == b - y
            && fits(self.schema.count(left, "atomEnd")? - x, 0, len)
            && fits(self.schema.count(right, "atomEnd")? - y, 0, len))
    }
    fn witness(&self, id: &str, node: Node<'a>) -> Result<(), Failure<'m>> {
        let left_id = self.schema.reference(node, "left")?;
        let right_id = self.schema.reference(node, "right")?;
        let left = self.typed(left_id, "Region")?;
        let right = self.typed(right_id, "Region")?;
        let order = self.schema.meter.compare(
            self.schema.reference(left, "function")?,
            self.schema.reference(right, "function")?,
        )?;
        let left_range = (
            self.schema.count(left, "atomStart")?,
            self.schema.count(left, "atomEnd")?,
        );
        let right_range = (
            self.schema.count(right, "atomStart")?,
            self.schema.count(right, "atomEnd")?,
        );
        if order.is_gt() || order.is_eq() && left_range >= right_range {
            return Err(self.error(E::Inconsistent, &"noncanonical witness endpoint order"));
        }
        let lf = self.target(left, "function", "Function")?;
        let rf = self.target(right, "function", "Function")?;
        let length = left_range.1 - left_range.0;
        if length < policy::MIN_LEXICAL
            || length != right_range.1 - right_range.0
            || self.eq(
                self.schema.reference(lf, "source")?,
                self.schema.reference(rf, "source")?,
            )? && self.schema.count(left, "startByte")? < self.schema.count(right, "endByte")?
                && self.schema.count(right, "startByte")? < self.schema.count(left, "endByte")?
        {
            return Err(self.error(E::Inconsistent, &"witness lengths or physical overlap"));
        }
        let expected = self.schema.key(
            "witness",
            [left_id, right_id, self.schema.digest(node, "digest")?],
        )?;
        if !self.eq(id, &expected)? {
            return Err(self.error(E::Name, &id));
        }
        let support = self.schema.reference(node, "candidate")?;
        if !self.supports(self.typed(support, "Candidate")?, left, right)? {
            return Err(self.error(
                E::Inconsistent,
                &"witness is outside its supporting coarse alignment",
            ));
        }
        for &value in self
            .schema
            .values(self.typed("", "EmbeddedReport")?, "candidate")?
            .iter()
        {
            let other = self
                .schema
                .iri(value)?
                .ok_or_else(|| self.error(E::Value, &"candidate reference required"))?;
            if self.schema.meter.compare(other, support)?.is_lt()
                && self.supports(self.typed(other, "Candidate")?, left, right)?
            {
                return Err(self.error(E::Inconsistent, &"witness support is not canonical"));
            }
        }
        Ok(())
    }
    fn reach(&self) -> Result<BorrowedSet<'m, 'a>, Failure<'m>> {
        let root = self.typed("", "EmbeddedReport")?;
        let mut seen = BorrowedSet::new(self.schema.meter);
        seen.insert("")?;
        for (field, expected) in [
            ("function", "Function"),
            ("candidate", "Candidate"),
            ("witness", "Witness"),
        ] {
            for &value in self.schema.values(root, field)?.iter() {
                let id = self
                    .schema
                    .iri(value)?
                    .ok_or_else(|| self.error(E::Value, &"owned report reference required"))?;
                let node = self.typed(id, expected)?;
                seen.insert(id)?;
                if expected == "Function" {
                    seen.insert(self.schema.reference(node, "source")?)?;
                    for &owner in self.schema.values(node, "owner")?.iter() {
                        seen.insert(
                            self.schema
                                .iri(owner)?
                                .ok_or_else(|| self.error(E::Value, &"owner reference required"))?,
                        )?;
                    }
                }
                if expected == "Witness" {
                    for field in ["left", "right"] {
                        seen.insert(self.schema.reference(node, field)?)?;
                    }
                }
            }
        }
        Ok(seen)
    }
    fn validate(&self) -> Result<(), Failure<'m>> {
        let nodes = self.schema.nodes(self.graph)?;
        for &(id, node) in nodes.iter() {
            self.shape(node)?;
            let class = self.schema.class(node)?;
            self.schema.meter.work(class.len())?;
            match class {
                "EmbeddedReport" if id.is_empty() => self.root(node)?,
                "Source" => self.source(id, node)?,
                "Function" => self.function(id, node)?,
                "Owner" => self.owner(id, node)?,
                "Region" => self.region(id, node)?,
                "Candidate" => self.candidate(id, node)?,
                "Witness" => {
                    self.schema.digest(node, "digest")?;
                }
                _ => {
                    return Err(
                        self.error(E::Class, &"embedded root must be the unique report root")
                    );
                }
            }
        }
        let reachable = self.reach()?;
        for &(id, node) in nodes.iter() {
            if !reachable.contains(id)? {
                return Err(self.error(E::Orphan, &id));
            }
            if self.eq(self.schema.class(node)?, "Witness")? {
                self.witness(id, node)?;
            }
        }
        Ok(())
    }
}
fn fits(start: usize, len: usize, end: usize) -> bool {
    start.checked_add(len).is_some_and(|limit| limit <= end)
}
pub(super) fn validate<'m>(graph: &Graph<'m>, meter: &'m Meter) -> Result<(), Failure<'m>> {
    Reader {
        graph,
        schema: Schema {
            meter,
            vocab: VOCAB,
            label: "embedded",
        },
    }
    .validate()
}
