// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Strict package/edge profile over the supplied native TOML arena.

use purrdf_hash::hex;
use purrdf_lex::allocation::{Admission, Memory};
use purrdf_lex::walk::{try_equal_range_by, try_sort_unstable_by};

use super::{BuildError, LockedPackage, LockedPackageId, LockedSource, LockedSourceKind, invalid};
use crate::manifest::native::{self, field as get};
use crate::meter::Meter;
use crate::{LoadError, RelPath};
use purrdf_toml::{Document, Kind, Node, Scalar};

type Rows = (Vec<LockedPackage>, Vec<LockedPackageId>);

pub(super) fn catalog<S: Admission + ?Sized>(
    text: &str,
    memory: &mut Memory<'_, S>,
    meter: &Meter,
    path: &RelPath,
) -> Result<Rows, BuildError> {
    let document = native::read(text).map_err(|source| {
        match memory.format(&format_args!("invalid TOML: {source}")) {
            Ok(message) => BuildError::Invalid {
                field: "lock",
                message,
                source: Some(Box::new(source)),
            },
            Err(error) => BuildError::Storage(error),
        }
    })?;
    let raw = package_rows(&document, memory, meter, path)?;
    let mut packages = Vec::new();
    memory.reserve(&mut packages, raw.len())?;
    for (index, value) in raw.iter().enumerate() {
        meter.work(1, path, None)?;
        let row = table(&document.nodes()[*value], memory)?;
        keys(
            &document,
            row,
            &["name", "version", "source", "checksum", "dependencies"],
            "package",
            memory,
            meter,
            path,
        )?;
        let package = package(&document, row, LockedPackageId(index), memory)?;
        memory.push(&mut packages, package)?;
    }
    let order = package_order(&packages, memory, meter, path)?;
    for (index, value) in raw.iter().enumerate() {
        let references =
            dependencies(&document, table(&document.nodes()[*value], memory)?, memory)?;
        let mut selected = Vec::new();
        memory.reserve(&mut selected, references.len())?;
        for value in references {
            meter.work(1, path, None)?;
            let Some(text) = native::string(&document.nodes()[*value]) else {
                return Err(invalid(
                    memory,
                    "package.dependencies",
                    format_args!("each dependency must be a string"),
                ));
            };
            let reference = Reference::parse(text, memory)?;
            let selection = Selection {
                packages: &packages,
                order: &order,
                from: index,
                meter,
                path,
            };
            let target = reference.select(&selection, text, memory)?;
            reference.release(memory)?;
            memory.push(&mut selected, target)?;
        }
        try_sort_unstable_by(&mut selected, |left, right| {
            meter.work(1, path, None)?;
            Ok::<_, LoadError>(left.cmp(right))
        })?;
        for pair in selected.windows(2) {
            meter.work(1, path, None)?;
            if pair[0] == pair[1] {
                return Err(invalid(
                    memory,
                    "package.dependencies",
                    format_args!(
                        "{} {} repeats outgoing package row {}",
                        packages[index].name,
                        packages[index].version,
                        pair[0].index()
                    ),
                ));
            }
        }
        packages[index].dependencies = selected;
    }
    Ok((packages, order))
}

fn package_rows<'a, S: Admission + ?Sized>(
    document: &'a Document<'_>,
    memory: &mut Memory<'_, S>,
    meter: &Meter,
    path: &RelPath,
) -> Result<&'a [usize], BuildError> {
    let root = &document.nodes()[0];
    keys(
        document,
        root,
        &["version", "package"],
        "lock",
        memory,
        meter,
        path,
    )?;
    if !get(document, root, "version")
        .is_some_and(|node| matches!(node.kind, Kind::Scalar(Scalar::Integer(3 | 4))))
    {
        return Err(invalid(
            memory,
            "version",
            format_args!("supported lock profile versions are 3 and 4"),
        ));
    }
    match get(document, root, "package") {
        None => Ok(&[]),
        Some(Node {
            kind: Kind::Array | Kind::ArrayOfTables,
            children: rows,
            ..
        }) => Ok(rows.as_slice()),
        Some(_) => Err(invalid(
            memory,
            "package",
            format_args!("must be an array of package tables"),
        )),
    }
}

fn package_order<S: Admission + ?Sized>(
    packages: &[LockedPackage],
    memory: &mut Memory<'_, S>,
    meter: &Meter,
    path: &RelPath,
) -> Result<Vec<LockedPackageId>, BuildError> {
    let mut order = Vec::new();
    memory.reserve(&mut order, packages.len())?;
    for index in 0..packages.len() {
        memory.push(&mut order, LockedPackageId(index))?;
    }
    try_sort_unstable_by(&mut order, |left, right| {
        meter.work(1, path, None)?;
        Ok::<_, LoadError>(
            packages[left.index()]
                .key()
                .cmp(&packages[right.index()].key()),
        )
    })?;
    for pair in order.windows(2) {
        meter.work(1, path, None)?;
        let first = &packages[pair[0].index()];
        let second = &packages[pair[1].index()];
        if first.key() == second.key() {
            return Err(invalid(
                memory,
                "package",
                format_args!(
                    "duplicate complete package identity for {} {}; rows {} and {}",
                    first.name,
                    first.version,
                    first.id.index(),
                    second.id.index()
                ),
            ));
        }
    }
    Ok(order)
}

fn table<'a, S: Admission + ?Sized>(
    value: &'a Node,
    memory: &mut Memory<'_, S>,
) -> Result<&'a Node, BuildError> {
    native::table(value).ok_or_else(|| {
        invalid(
            memory,
            "package",
            format_args!("each package must be a table"),
        )
    })
}

fn keys<S: Admission + ?Sized>(
    document: &Document<'_>,
    table: &Node,
    allowed: &[&str],
    field: &'static str,
    memory: &mut Memory<'_, S>,
    meter: &Meter,
    path: &RelPath,
) -> Result<(), BuildError> {
    for (key, _) in native::entries(document, table) {
        meter.work(1, path, None)?;
        if !allowed.contains(&key) {
            return Err(invalid(
                memory,
                field,
                format_args!(
                    "unsupported field {key:?}; replacements and other lock profiles require explicit support"
                ),
            ));
        }
    }
    Ok(())
}

fn string<'a, S: Admission + ?Sized>(
    document: &'a Document<'_>,
    table: &Node,
    key: &'static str,
    memory: &mut Memory<'_, S>,
) -> Result<&'a str, BuildError> {
    get(document, table, key)
        .and_then(native::string)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| invalid(memory, key, format_args!("a nonempty string is required")))
}

fn package<S: Admission + ?Sized>(
    document: &Document<'_>,
    row: &Node,
    id: LockedPackageId,
    memory: &mut Memory<'_, S>,
) -> Result<LockedPackage, BuildError> {
    let raw_name = string(document, row, "name", memory)?;
    let name = memory.string(raw_name)?;
    let raw_version = string(document, row, "version", memory)?;
    let version = raw_version.parse::<semver::Version>().map_err(|error| {
        invalid(
            memory,
            "version",
            format_args!("invalid package version: {error}"),
        )
    })?;
    let version = memory.format(&format_args!("{version}"))?;
    let source = get(document, row, "source")
        .map(|value| {
            let text = native::string(value)
                .ok_or_else(|| invalid(memory, "source", format_args!("must be a string")))?;
            LockedSource::parse(text, memory)
        })
        .transpose()?;
    if source
        .as_ref()
        .is_some_and(|source| source.kind() == LockedSourceKind::Git && source.revision().is_none())
    {
        return Err(invalid(
            memory,
            "source",
            format_args!("a Git package must record its full selected revision"),
        ));
    }
    let checksum = get(document, row, "checksum")
        .map(|value| {
            native::string(value)
                .and_then(hex::decode_32)
                .ok_or_else(|| {
                    invalid(
                        memory,
                        "checksum",
                        format_args!("must be exactly 64 ASCII hexadecimal digits"),
                    )
                })
        })
        .transpose()?;
    Ok(LockedPackage {
        id,
        name,
        version,
        source,
        checksum,
        dependencies: Vec::new(),
    })
}

fn dependencies<'a, S: Admission + ?Sized>(
    document: &'a Document<'_>,
    row: &Node,
    memory: &mut Memory<'_, S>,
) -> Result<&'a [usize], BuildError> {
    let values = match get(document, row, "dependencies") {
        None => &[][..],
        Some(Node {
            kind: Kind::Array | Kind::ArrayOfTables,
            children: values,
            ..
        }) => values.as_slice(),
        Some(_) => {
            return Err(invalid(
                memory,
                "package.dependencies",
                format_args!("must be an array of dependency strings"),
            ));
        }
    };
    Ok(values)
}

struct Reference<'a> {
    name: &'a str,
    version: Option<String>,
    source: Option<LockedSource>,
}

struct Selection<'a> {
    packages: &'a [LockedPackage],
    order: &'a [LockedPackageId],
    from: usize,
    meter: &'a Meter,
    path: &'a RelPath,
}

impl<'a> Reference<'a> {
    fn parse<S: Admission + ?Sized>(
        text: &'a str,
        memory: &mut Memory<'_, S>,
    ) -> Result<Self, BuildError> {
        let (head, source) = match text.split_once(" (") {
            Some((head, suffix)) => {
                let source = suffix.strip_suffix(')').ok_or_else(|| {
                    invalid(
                        memory,
                        "package.dependencies",
                        format_args!("source qualification must end in ')'"),
                    )
                })?;
                (head, Some(LockedSource::parse(source, memory)?))
            }
            None => (text, None),
        };
        let mut words = head.split_ascii_whitespace();
        let name = words.next().ok_or_else(|| {
            invalid(
                memory,
                "package.dependencies",
                format_args!("dependency name is required"),
            )
        })?;
        let version = words
            .next()
            .map(|text| {
                let version = text.parse::<semver::Version>().map_err(|error| {
                    invalid(
                        memory,
                        "package.dependencies",
                        format_args!("invalid dependency version: {error}"),
                    )
                })?;
                Ok::<_, BuildError>(memory.format(&format_args!("{version}"))?)
            })
            .transpose()?;
        if words.next().is_some() {
            return Err(invalid(
                memory,
                "package.dependencies",
                format_args!("expected name, optional version and optional source qualification"),
            ));
        }
        Ok(Self {
            name,
            version,
            source,
        })
    }

    fn select<S: Admission + ?Sized>(
        &self,
        selection: &Selection<'_>,
        text: &str,
        memory: &mut Memory<'_, S>,
    ) -> Result<LockedPackageId, BuildError> {
        let Selection {
            packages,
            order,
            from,
            meter,
            path,
        } = *selection;
        let range = try_equal_range_by(order, |id| {
            meter.work(1, path, None)?;
            Ok::<_, LoadError>(packages[id.index()].name.as_str().cmp(self.name))
        })?;
        let mut selected = None;
        for id in &order[range] {
            meter.work(1, path, None)?;
            let package = &packages[id.index()];
            if self
                .version
                .as_ref()
                .is_some_and(|version| *version != package.version)
                || self
                    .source
                    .as_ref()
                    .is_some_and(|source| Some(source) != package.source.as_ref())
            {
                continue;
            }
            if selected.is_some() {
                return Err(invalid(
                    memory,
                    "package.dependencies",
                    format_args!(
                        "{} {}: reference {text:?} is ambiguous; qualify its complete version and source",
                        packages[from].name, packages[from].version
                    ),
                ));
            }
            selected = Some(*id);
        }
        selected.ok_or_else(|| invalid(memory, "package.dependencies", format_args!("{} {}: reference {text:?} selects no complete locked package; regenerate the lock from its declarations", packages[from].name, packages[from].version)))
    }

    fn release<S: Admission + ?Sized>(self, memory: &mut Memory<'_, S>) -> Result<(), BuildError> {
        if let Some(version) = self.version {
            memory.release_string(version)?;
        }
        if let Some(source) = self.source {
            source.release(memory)?;
        }
        Ok(())
    }
}
