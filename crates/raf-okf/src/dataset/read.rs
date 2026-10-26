// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed native-index reads, then existing RAF reconstruction and validation.

use purrdf_core::{GraphMatch, QuadIds, RdfDataset, TermId, TermRef};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};

use super::{
    Budget, DOCUMENT_IRI_BASE, DatasetError as E, INTEGER, MAX_DATASET_NODES, STRING, TYPE, VOCAB,
    descendants, identity, property, validate,
};
use crate::{
    Comments, Document, Entry, Frontmatter, ItemValue, List, ListForm, ListItem, Map, Scalar,
    ScalarStyle, Value,
};

type Set = OrderedTable<TermId, ()>;

fn insert(set: &mut Set, id: TermId) -> Result<bool, E> {
    let mut resident = Resident;
    Ok(set
        .insert(id, (), &mut Memory::new(&mut resident))
        .map_err(E::native)?
        .is_none())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Document,
    Map,
    Entry,
    Scalar,
    List,
    Item,
    Comment,
}

impl Role {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "Document" => Self::Document,
            "Map" => Self::Map,
            "Entry" => Self::Entry,
            "Scalar" => Self::Scalar,
            "List" => Self::List,
            "Item" => Self::Item,
            "Comment" => Self::Comment,
            _ => return None,
        })
    }
    fn fields(self) -> &'static [(&'static str, usize, usize)] {
        const MANY: usize = MAX_DATASET_NODES;
        match self {
            Self::Document => &[
                ("frontmatter", 1, 1),
                ("body", 1, 1),
                ("trailingComment", 0, MANY),
            ],
            Self::Map => &[("entry", 1, MANY)],
            Self::Entry => &[
                ("ordinal", 1, 1),
                ("key", 1, 1),
                ("value", 1, 1),
                ("leadingComment", 0, MANY),
                ("trailing", 0, 1),
            ],
            Self::Scalar => &[("text", 1, 1), ("style", 1, 1)],
            Self::List => &[("form", 1, 1), ("item", 0, MANY)],
            Self::Item => &[
                ("ordinal", 1, 1),
                ("value", 1, 1),
                ("headEntry", 0, 1),
                ("leadingComment", 0, MANY),
                ("trailing", 0, 1),
            ],
            Self::Comment => &[("ordinal", 1, 1), ("text", 1, 1)],
        }
    }
}

struct Reader<'a> {
    dataset: &'a RdfDataset,
    subjects: Set,
    seen: Set,
}

pub(super) fn document(dataset: &RdfDataset, path: &str) -> Result<Document, E> {
    let mut reader = Reader::new(dataset)?;
    let roots: Vec<_> = reader
        .subjects
        .iter()
        .filter_map(|(id, ())| (reader.role(*id).ok() == Some(Role::Document)).then_some(*id))
        .collect();
    if roots.len() != 1 {
        return Err(E::shape(
            "carrier",
            "exactly one Document subject is required",
        ));
    }
    let root = roots[0];
    let root_iri = reader.iri(root)?;
    if !root_iri.starts_with(DOCUMENT_IRI_BASE)
        || root_iri.len() > crate::MAX_TEXT_BYTES + DOCUMENT_IRI_BASE.len() + 64
    {
        return Err(E::shape(
            "document",
            "document identity is outside the stable type/id namespace",
        ));
    }
    let prefix = descendants(root_iri);
    for (subject, ()) in reader.subjects.iter() {
        if *subject != root {
            let suffix = reader.iri(*subject)?.strip_prefix(&prefix).ok_or_else(|| {
                E::shape(
                    reader.iri(*subject).unwrap_or("node"),
                    "unrelated structural identity",
                )
            })?;
            let ordinal = suffix
                .parse::<usize>()
                .ok()
                .filter(|n| n.to_string() == suffix && *n < reader.subjects.len() - 1);
            if ordinal.is_none() {
                return Err(E::shape(
                    "node",
                    "structural suffix must be a bounded contiguous ordinal",
                ));
            }
        }
        reader.shape(*subject)?;
    }
    reader.enter(root, Role::Document)?;
    let front = reader.one(root, "frontmatter")?;
    let map = reader.map(front, 1, None)?;
    let trailing_comments = reader.comments_at(root, "trailingComment")?;
    let frontmatter = Frontmatter {
        map,
        trailing_comments,
    };
    let body = reader.string(root, "body")?.to_owned();
    if reader.seen.len() != reader.subjects.len() {
        return Err(E::shape(
            root_iri,
            "unrelated or orphan structural subjects",
        ));
    }
    let document = Document::new(frontmatter.clone(), body).map_err(E::Construct)?;
    if document.frontmatter() != &frontmatter {
        return Err(E::shape(
            root_iri,
            "representation changes under document normalization",
        ));
    }
    let parsed = Document::parse(&document.render()).map_err(E::Parse)?;
    if parsed != document {
        return Err(E::shape(
            root_iri,
            "styles, comments or values do not survive RAF rendering",
        ));
    }
    validate(&parsed, path)?;
    if identity(&parsed)? != root_iri {
        return Err(E::shape(
            root_iri,
            "asserted identity disagrees with validated type/id",
        ));
    }
    Ok(parsed)
}

impl<'a> Reader<'a> {
    /// Check all counts and authoring lexical bytes before cloning any value.
    fn new(dataset: &'a RdfDataset) -> Result<Self, E> {
        if dataset.named_graphs().next().is_some()
            || dataset.reifiers().next().is_some()
            || dataset.annotations().next().is_some()
        {
            return Err(E::shape(
                "carrier",
                "named graphs and RDF statement layers are outside the authoring shape",
            ));
        }
        let mut budget = Budget::default();
        let mut subjects = Set::default();
        for quad in dataset.quads() {
            budget.triple()?;
            if quad.g.is_some() {
                return Err(E::shape("carrier", "named graph row"));
            }
            if !matches!(dataset.resolve(quad.s), TermRef::Iri(_)) {
                return Err(E::shape("carrier", "every subject must be an IRI"));
            }
            if insert(&mut subjects, quad.s)? {
                budget.node()?;
            }
            let TermRef::Iri(predicate) = dataset.resolve(quad.p) else {
                return Err(E::shape("carrier", "predicate IRI required"));
            };
            match dataset.resolve(quad.o) {
                TermRef::Iri(_) => {}
                TermRef::Literal {
                    lexical,
                    datatype,
                    language: None,
                    direction: None,
                } => {
                    let expected = if predicate == property("ordinal") {
                        INTEGER
                    } else {
                        STRING
                    };
                    if !matches!(dataset.resolve(datatype), TermRef::Iri(value) if value == expected)
                    {
                        return Err(E::shape(
                            "carrier",
                            "canonical integer or string datatype required",
                        ));
                    }
                    if ["body", "key", "text", "trailing"]
                        .iter()
                        .any(|name| predicate == property(name))
                    {
                        budget.text(lexical)?;
                    }
                }
                _ => {
                    return Err(E::shape(
                        "carrier",
                        "blank nodes, quoted terms and language metadata are unsupported",
                    ));
                }
            }
        }
        Ok(Self {
            dataset,
            subjects,
            seen: Set::default(),
        })
    }

    fn rows(&self, id: TermId) -> impl Iterator<Item = QuadIds> + '_ {
        self.dataset.quads_for_pattern_with_plan(
            &RdfDataset::probe_plan(true, false, false, GraphMatch::Default),
            Some(id),
            None,
            None,
            GraphMatch::Default,
        )
    }

    fn values(&self, id: TermId, key: &str) -> impl Iterator<Item = TermId> + '_ {
        let predicate = self.dataset.term_id_by_iri(&if key == "type" {
            TYPE.into()
        } else {
            property(key)
        });
        self.rows(id)
            .filter(move |quad| Some(quad.p) == predicate)
            .map(|quad| quad.o)
    }

    fn iri(&self, id: TermId) -> Result<&'a str, E> {
        if let TermRef::Iri(iri) = self.dataset.resolve(id) {
            Ok(iri)
        } else {
            Err(E::shape("node", "IRI required"))
        }
    }

    fn one(&self, id: TermId, key: &str) -> Result<TermId, E> {
        let mut values = self.values(id, key);
        match (values.next(), values.next()) {
            (Some(value), None) => Ok(value),
            _ => Err(E::shape(
                self.iri(id)?,
                format!("exactly one {key} is required"),
            )),
        }
    }

    fn string(&self, id: TermId, key: &str) -> Result<&'a str, E> {
        if let TermRef::Literal {
            lexical,
            datatype,
            language: None,
            direction: None,
        } = self.dataset.resolve(self.one(id, key)?)
            && matches!(self.dataset.resolve(datatype), TermRef::Iri(value) if value == STRING)
        {
            Ok(lexical)
        } else {
            Err(E::shape(self.iri(id)?, format!("{key} requires a string")))
        }
    }

    fn role(&self, id: TermId) -> Result<Role, E> {
        self.iri(self.one(id, "type")?)?
            .strip_prefix(VOCAB)
            .and_then(Role::named)
            .ok_or_else(|| E::shape(self.iri(id).unwrap_or("node"), "unknown authoring role"))
    }

    fn shape(&self, id: TermId) -> Result<(), E> {
        let role = self.role(id)?;
        for quad in self.rows(id) {
            let predicate = self.iri(quad.p)?;
            if predicate == TYPE {
                continue;
            }
            let key = predicate.strip_prefix(VOCAB).ok_or_else(|| {
                E::shape(
                    self.iri(id).unwrap_or("node"),
                    "unknown predicate namespace",
                )
            })?;
            if !role.fields().iter().any(|(name, _, _)| *name == key) {
                return Err(E::shape(
                    self.iri(id)?,
                    "unknown predicate on this role; use only the declared authoring fields",
                ));
            }
            if [
                "frontmatter",
                "entry",
                "value",
                "item",
                "headEntry",
                "leadingComment",
                "trailingComment",
            ]
            .contains(&key)
            {
                if !matches!(self.dataset.resolve(quad.o), TermRef::Iri(_))
                    || self.subjects.get(&quad.o).is_none()
                {
                    return Err(E::shape(
                        self.iri(id)?,
                        format!("{key} requires an asserted structural subject"),
                    ));
                }
            } else if key == "ordinal" {
                self.ordinal(id)?;
            } else {
                self.string(id, key)?;
            }
        }
        for (key, min, max) in role.fields() {
            let count = self.values(id, key).count();
            if count < *min || count > *max {
                return Err(E::shape(
                    self.iri(id)?,
                    format!("{key} cardinality must be {min}..{max}"),
                ));
            }
        }
        Ok(())
    }

    fn enter(&mut self, id: TermId, role: Role) -> Result<(), E> {
        if self.role(id)? != role {
            return Err(E::shape(self.iri(id)?, "wrong structural role"));
        }
        if !insert(&mut self.seen, id)? {
            return Err(E::shape(self.iri(id)?, "cycle or shared structural child"));
        }
        Ok(())
    }

    fn ordinal(&self, id: TermId) -> Result<usize, E> {
        if let TermRef::Literal {
            lexical,
            datatype,
            language: None,
            direction: None,
        } = self.dataset.resolve(self.one(id, "ordinal")?)
            && matches!(self.dataset.resolve(datatype), TermRef::Iri(value) if value == INTEGER)
            && lexical.len() <= 6
            && let Ok(value) = lexical.parse::<usize>()
            && value < MAX_DATASET_NODES
            && value.to_string() == lexical
        {
            Ok(value)
        } else {
            Err(E::shape(
                self.iri(id)?,
                "bounded canonical nonnegative ordinal required",
            ))
        }
    }

    fn ordered(&self, id: TermId, key: &str) -> Result<Vec<TermId>, E> {
        let mut children = self
            .values(id, key)
            .map(|child| Ok((self.ordinal(child)?, child)))
            .collect::<Result<Vec<_>, E>>()?;
        purrdf_lex::walk::try_sort_unstable_by(&mut children, |a, b| Ok::<_, E>(a.0.cmp(&b.0)))?;
        if children
            .iter()
            .enumerate()
            .any(|(expected, (ordinal, _))| expected != *ordinal)
        {
            return Err(E::shape(
                self.iri(id)?,
                format!("{key} ordinals repeat or have gaps"),
            ));
        }
        Ok(children.into_iter().map(|(_, child)| child).collect())
    }

    fn comments_at(&mut self, id: TermId, key: &str) -> Result<Vec<String>, E> {
        self.ordered(id, key)?
            .into_iter()
            .map(|child| {
                self.enter(child, Role::Comment)?;
                Ok(self.string(child, "text")?.into())
            })
            .collect()
    }

    fn comments(&mut self, id: TermId) -> Result<Comments, E> {
        let leading = self.comments_at(id, "leadingComment")?;
        let trailing = self
            .values(id, "trailing")
            .next()
            .map(|_| self.string(id, "trailing").map(str::to_owned))
            .transpose()?;
        Ok(Comments { leading, trailing })
    }

    fn map(&mut self, id: TermId, depth: usize, head: Option<TermId>) -> Result<Map, E> {
        if depth > crate::MAX_YAML_DEPTH {
            return Err(E::Limit("32 collection levels"));
        }
        self.enter(id, Role::Map)?;
        let ids = self.ordered(id, "entry")?;
        let mut entries = Vec::with_capacity(ids.len());
        for child in &ids {
            self.enter(*child, Role::Entry)?;
            let key = self.string(*child, "key")?.to_owned();
            let comments = self.comments(*child)?;
            let value = self.value(self.one(*child, "value")?, depth + 1)?;
            let mut entry = Entry::new(key, value).map_err(E::Construct)?;
            entry.comments = comments;
            entries.push(entry);
        }
        if entries
            .windows(2)
            .any(|pair| pair[0].key() >= pair[1].key())
        {
            return Err(E::shape(
                self.iri(id)?,
                "mapping ordinals must follow unique sorted keys",
            ));
        }
        if let Some(head) = head {
            let position = ids.iter().position(|child| *child == head).ok_or_else(|| {
                E::shape(
                    self.iri(id).unwrap_or("map"),
                    "headEntry is not a child of this mapping",
                )
            })?;
            let first = entries.remove(position);
            entries.insert(0, first);
        }
        Map::new(entries).map_err(E::Construct)
    }

    fn value(&mut self, id: TermId, depth: usize) -> Result<Value, E> {
        match self.role(id)? {
            Role::Scalar => {
                self.enter(id, Role::Scalar)?;
                let mut value = Scalar::new(self.string(id, "text")?).map_err(E::Construct)?;
                value.style = match self.string(id, "style")? {
                    "plain" => ScalarStyle::Plain,
                    "single" => ScalarStyle::SingleQuoted,
                    "double" => ScalarStyle::DoubleQuoted,
                    _ => return Err(E::shape(self.iri(id)?, "unknown scalar style")),
                };
                Ok(Value::Scalar(value))
            }
            Role::Map => self.map(id, depth, None).map(Value::Map),
            Role::List => {
                if depth > crate::MAX_YAML_DEPTH {
                    return Err(E::Limit("32 collection levels"));
                }
                self.enter(id, Role::List)?;
                let form = match self.string(id, "form")? {
                    "flow" => ListForm::Flow,
                    "block" => ListForm::Block,
                    _ => return Err(E::shape(self.iri(id)?, "unknown list form")),
                };
                let mut items = Vec::new();
                for child in self.ordered(id, "item")? {
                    self.enter(child, Role::Item)?;
                    let comments = self.comments(child)?;
                    let value = self.one(child, "value")?;
                    let head = self.values(child, "headEntry").next();
                    let value = match (self.role(value)?, head) {
                        (Role::Map, Some(head)) => {
                            ItemValue::Map(self.map(value, depth + 1, Some(head))?)
                        }
                        (Role::Scalar, None) => {
                            let Value::Scalar(value) = self.value(value, depth + 1)? else {
                                unreachable!("scalar role was checked");
                            };
                            ItemValue::Scalar(value)
                        }
                        _ => {
                            return Err(E::shape(
                                self.iri(child)?,
                                "item requires a scalar, or mapping with exactly its headEntry",
                            ));
                        }
                    };
                    items.push(ListItem {
                        value,
                        comments,
                        line: 0,
                    });
                }
                let list = match form {
                    ListForm::Block => List::block(items).map_err(E::Construct)?,
                    ListForm::Flow => {
                        if items.iter().any(|item| {
                            !matches!(item.value, ItemValue::Scalar(_))
                                || item.comments != Comments::default()
                        }) {
                            return Err(E::shape(
                                self.iri(id)?,
                                "flow lists require scalar items without item comments",
                            ));
                        }
                        let scalars = items
                            .into_iter()
                            .filter_map(|item| {
                                if let ItemValue::Scalar(scalar) = item.value {
                                    Some(scalar)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        List::flow(scalars).map_err(E::Construct)?
                    }
                };
                Ok(Value::List(list))
            }
            _ => Err(E::shape(self.iri(id)?, "value must be Scalar, Map or List")),
        }
    }
}
