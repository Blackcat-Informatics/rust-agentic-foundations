// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declared source/owner descriptors for location-independent counted debt.
//!
//! Descriptors are policy keys, not persistent physical occurrence identities.

use super::analysis::{Function, Origin};
use super::debt::Failure;
use super::resource::{Grant, Meter, Refusal, Scratch, Text};
use crate::report::ReportError as E;
use raf_source::{
    Item, LockedGitReference, LockedSource, LockedSourceKind, Package, SourceAuthorityId,
    SymbolKind, Target, TargetKind, TraitKey, Workspace,
};

pub(super) struct Owner<'a> {
    pub(super) item: &'a Item,
    pub(super) target: &'a Target,
    pub(super) package: &'a Package,
}

pub(super) fn owner<'a>(
    workspace: &'a Workspace,
    item: &'a Item,
) -> Result<Owner<'a>, &'static str> {
    if !workspace
        .item(item.id)
        .is_some_and(|actual| std::ptr::eq(actual, item))
    {
        return Err("owner is not an actual indexed item");
    }
    let target = workspace
        .target(item.target)
        .ok_or("owner lost its actual target")?;
    let package = workspace
        .package(target.package_id)
        .ok_or("owner target lost its package")?;
    Ok(Owner {
        item,
        target,
        package,
    })
}

pub(super) fn target_kind(kind: TargetKind) -> Result<&'static str, &'static str> {
    Ok(match kind {
        TargetKind::Lib => "lib",
        TargetKind::Bin => "bin",
        TargetKind::Test => "test",
        TargetKind::Example => "example",
        TargetKind::Bench => "bench",
        TargetKind::BuildScript => "build-script",
        _ => return Err("unmodeled source target kind"),
    })
}

fn key<'m, 'a>(
    meter: &'m Meter,
    kind: &str,
    parts: impl IntoIterator<Item = &'a str>,
) -> Result<Text<'m>, Failure<'m>> {
    Ok(Text::build(meter, |memory| {
        Ok(super::report::key_with_memory(kind, parts, memory)?)
    })?)
}

fn source_kind(kind: LockedSourceKind) -> &'static str {
    match kind {
        LockedSourceKind::Git => "git",
        LockedSourceKind::Path => "path",
        LockedSourceKind::Directory => "directory",
        LockedSourceKind::Registry => "registry",
        LockedSourceKind::SparseRegistry => "sparse",
        LockedSourceKind::LocalRegistry => "local-registry",
    }
}

fn external<'m>(
    meter: &'m Meter,
    role: &str,
    name: &str,
    version: &str,
    source: Option<&LockedSource>,
) -> Result<Text<'m>, Failure<'m>> {
    let source = source.ok_or_else(|| {
        Failure::report(
            meter,
            E::Value,
            &"embedded debt needs an exact native source binding for an external root",
        )
    })?;
    let (reference_kind, reference) = match source.reference() {
        None => ("absent", ""),
        Some(LockedGitReference::DefaultBranch) => ("default-branch", ""),
        Some(LockedGitReference::Branch(text)) => ("branch", text.as_str()),
        Some(LockedGitReference::Tag(text)) => ("tag", text.as_str()),
        Some(LockedGitReference::Revision(text)) => ("revision", text.as_str()),
    };
    key(
        meter,
        "source-v1",
        [
            role,
            name,
            version,
            source_kind(source.kind()),
            source.repository(),
            reference_kind,
            reference,
            if source.revision().is_some() {
                "present"
            } else {
                "absent"
            },
            source.revision().unwrap_or(""),
        ],
    )
}

fn qualification<'m>(
    meter: &'m Meter,
    workspace: &Workspace,
    target: &Target,
    package: &Package,
) -> Result<Text<'m>, Failure<'m>> {
    if target.root.authority() != package.manifest.authority() {
        return Err(Failure::report(
            meter,
            E::Inconsistent,
            &"target and manifest have different source authorities",
        ));
    }
    if package.manifest.authority() == SourceAuthorityId::FIRST_PARTY {
        return Ok(Text::new(meter, "first-party-root")?);
    }
    if let Some(binding) = workspace.locked_binding(target.package_id) {
        return external(
            meter,
            "configured-locked",
            binding.name(),
            binding.version(),
            binding.source(),
        );
    }
    if let Some(binding) = workspace.reference_binding(target.package_id) {
        return external(
            meter,
            "structural-observation",
            binding.name(),
            binding.version(),
            binding.source(),
        );
    }
    Err(Failure::report(
        meter,
        E::Value,
        &"embedded debt cannot persist a load-local external authority; load its checked native source binding",
    ))
}

fn target_key<'m>(
    meter: &'m Meter,
    workspace: &Workspace,
    target: &Target,
) -> Result<Text<'m>, Failure<'m>> {
    let package = workspace
        .package(target.package_id)
        .ok_or_else(|| Failure::report(meter, E::Inconsistent, &"target lost its package"))?;
    let qualification = qualification(meter, workspace, target, package)?;
    key(
        meter,
        "target-v1",
        [
            &qualification,
            package.manifest.as_str(),
            &package.name,
            target_kind(target.kind).map_err(|reason| Failure::report(meter, E::Value, &reason))?,
            &target.name,
            target.root.as_str(),
            &target.crate_path,
        ],
    )
}

fn optional<'m>(meter: &'m Meter, value: Option<&str>) -> Result<Text<'m>, Failure<'m>> {
    match value {
        None => key(meter, "absent", []),
        Some(text) => key(meter, "present", [text]),
    }
}

fn trait_key<'m>(meter: &'m Meter, item: &Item) -> Result<Text<'m>, Failure<'m>> {
    match item.trait_key.as_deref() {
        None => key(meter, "absent", []),
        Some(TraitKey::Canonical(text)) => key(meter, "canonical", [text.as_str()]),
        Some(TraitKey::Unresolved {
            written,
            candidates,
        }) => {
            meter.work(candidates.len())?;
            key(
                meter,
                "unresolved",
                std::iter::once(written.as_str()).chain(candidates.iter().map(String::as_str)),
            )
        }
        Some(_) => Err(Failure::report(
            meter,
            E::Value,
            &"unmodeled native trait identity",
        )),
    }
}

fn descriptor<'m>(
    meter: &'m Meter,
    workspace: &Workspace,
    item: &Item,
) -> Result<Text<'m>, Failure<'m>> {
    meter.work(1)?;
    let owner = owner(workspace, item)
        .map_err(|reason| Failure::report(meter, E::Inconsistent, &reason))?;
    if item.file.authority() != owner.package.manifest.authority() {
        return Err(Failure::report(
            meter,
            E::Inconsistent,
            &"owner file and package have different source authorities",
        ));
    }
    let kind = match item.kind {
        SymbolKind::Fn => "function",
        SymbolKind::InherentMethod => "inherent-method",
        SymbolKind::TraitImplMethod => "trait-impl-method",
        SymbolKind::TraitDefaultMethod => "trait-default-method",
        _ => {
            return Err(Failure::report(
                meter,
                E::Value,
                &"embedded debt owner is not a function",
            ));
        }
    };
    let definition = if let Some(id) = item.trait_definition_target {
        let target = workspace.target(id).ok_or_else(|| {
            Failure::report(meter, E::Inconsistent, &"trait definition target is absent")
        })?;
        let target = target_key(meter, workspace, target)?;
        key(meter, "present", [&*target])?
    } else {
        key(meter, "absent", [])?
    };
    let target = target_key(meter, workspace, owner.target)?;
    let self_type = optional(meter, item.self_type.as_deref())?;
    let trait_path = optional(meter, item.trait_path.as_deref())?;
    let trait_key = trait_key(meter, item)?;
    key(
        meter,
        "owner-v1",
        [
            &*target,
            kind,
            item.path.as_str(),
            &self_type,
            &trait_path,
            &trait_key,
            &definition,
        ],
    )
}

pub(super) fn endpoint<'m>(
    function: &Function<'_>,
    meter: &'m Meter,
) -> Result<Text<'m>, Failure<'m>> {
    let (workspace, file, owners, profile) = match &function.origin {
        Origin::Source {
            workspace,
            file,
            owners,
            profile,
        } => (*workspace, *file, owners, *profile),
        #[cfg(test)]
        Origin::Text => {
            return Err(Failure::report(
                meter,
                E::Value,
                &"embedded debt requires actual source provenance",
            ));
        }
    };
    meter.work(file.path.as_str().len())?;
    if owners.is_empty()
        || !workspace
            .file(&file.path)
            .is_some_and(|actual| std::ptr::eq(actual, file))
    {
        return Err(Failure::report(
            meter,
            E::Inconsistent,
            &"embedded endpoint lost actual source ownership",
        ));
    }
    let mut identities = Scratch::new(meter);
    for item in owners {
        meter.work(
            item.file
                .as_str()
                .len()
                .min(file.path.as_str().len())
                .saturating_add(1),
        )?;
        if item.file != file.path
            || item.body.as_ref().is_none_or(|body| {
                body.start_byte != function.facts.body.start
                    || body.end_byte != function.facts.body.end
            })
        {
            return Err(Failure::report(
                meter,
                E::Inconsistent,
                &"endpoint owner body differs from source",
            ));
        }
        identities.push(descriptor(meter, workspace, item)?)?;
    }
    purrdf_lex::walk::try_sort_unstable_by(identities.as_mut_slice(), |a, b| meter.compare(a, b))?;
    let mut unique: Scratch<'_, &str> = Scratch::new(meter);
    for identity in identities.iter() {
        let duplicate = match unique.as_slice().last() {
            Some(prior) => meter.compare(prior, identity)?.is_eq(),
            None => false,
        };
        if !duplicate {
            unique.push(&**identity)?;
        }
    }
    let (disabled, no_std) = profile.flags();
    let edition = Text::display(meter, &profile.edition())?;
    key(
        meter,
        "endpoint-v1",
        [
            file.path.as_str(),
            &edition,
            if disabled { "true" } else { "false" },
            if no_std { "true" } else { "false" },
        ]
        .into_iter()
        .chain(unique.iter().copied()),
    )
}

pub(super) fn validate_endpoint<'m>(text: &str, meter: &'m Meter) -> Result<(), Failure<'m>> {
    let parts = components(text, "endpoint-v1", meter)?;
    if parts.len() < 5 {
        return Err(Failure::report(
            meter,
            E::Value,
            &"incomplete embedded endpoint descriptor",
        ));
    }
    canonical_path(&parts[0], false, meter)?;
    if !matches!(parts[1].as_ref(), "2015" | "2018" | "2021" | "2024")
        || !matches!(parts[2].as_ref(), "true" | "false")
        || !matches!(parts[3].as_ref(), "true" | "false")
    {
        return Err(Failure::report(
            meter,
            E::Value,
            &"invalid embedded endpoint policy key",
        ));
    }
    for owners in parts[4..].windows(2) {
        if !meter.compare(&owners[0], &owners[1])?.is_lt() {
            return Err(Failure::report(
                meter,
                E::Value,
                &"invalid embedded endpoint policy key",
            ));
        }
    }
    for owner in &parts[4..] {
        validate_owner(owner, meter)?;
    }
    Ok(())
}

fn components<'m>(
    text: &str,
    tag: &str,
    meter: &'m Meter,
) -> Result<Scratch<'m, Text<'m>>, Failure<'m>> {
    meter.work(text.len())?;
    let rest = text
        .strip_prefix('#')
        .and_then(|rest| rest.strip_prefix(tag))
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(|| {
            Failure::report(
                meter,
                E::Value,
                &format_args!("unsupported embedded {tag} descriptor"),
            )
        })?;
    let mut parts = Scratch::new(meter);
    for part in rest.split('/') {
        let decoded = Text::try_build(meter, |memory| -> Result<String, Failure<'m>> {
            let mut bytes = memory
                .string_with_progress(part)
                .map_err(Refusal::from)?
                .into_bytes();
            meter.work(bytes.len())?;
            let len = purrdf_lex::percent::decode_in_place(&mut bytes).map_err(Failure::Percent)?;
            bytes.truncate(len);
            // The native decoder has just validated this exact decoded prefix.
            Ok(String::from_utf8(bytes).expect("native percent decoder validated UTF-8"))
        })?;
        let encoded = Text::build(meter, |memory| {
            let encoded = crate::report::mint::part_with_memory(&decoded, memory)?;
            match encoded {
                std::borrow::Cow::Owned(text) => Ok(text),
                std::borrow::Cow::Borrowed(text) => Ok(memory.string_with_progress(text)?),
            }
        })?;
        if !meter.compare(&encoded, part)?.is_eq() {
            return Err(Failure::report(
                meter,
                E::Value,
                &format_args!("noncanonical embedded {tag} component"),
            ));
        }
        parts.push(decoded)?;
    }
    Ok(parts)
}

fn canonical_path<'m>(text: &str, manifest: bool, meter: &'m Meter) -> Result<(), Failure<'m>> {
    meter.work(text.len())?;
    let file_name = text.rsplit('/').next().filter(|name| !name.is_empty());
    if !raf_source::RelPath::is_canonical(text)
        || file_name.is_none()
        || manifest && file_name != Some("Cargo.toml")
    {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded descriptor requires canonical source-relative file spelling",
        ));
    }
    Ok(())
}

fn validate_repository<'m>(
    repository: &str,
    git: bool,
    meter: &'m Meter,
) -> Result<(), Failure<'m>> {
    let mut validation = Grant::new(meter);
    let absolute = validation.memory(|memory| {
        Ok(purrdf_iri::absolute_verdict_with_memory(
            repository, memory,
        )?)
    })?;
    meter.work(repository.len())?;
    let (query, fragment) = repository
        .bytes()
        .fold((false, false), |(query, fragment), byte| {
            (query || byte == b'?', fragment || byte == b'#')
        });
    if absolute != Some(true) || fragment || git && query {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded source repository must retain its native absolute IRI shape",
        ));
    }
    Ok(())
}

fn validate_source<'m>(text: &str, package: &str, meter: &'m Meter) -> Result<(), Failure<'m>> {
    if meter.compare(text, "first-party-root")?.is_eq() {
        return Ok(());
    }
    let parts = components(text, "source-v1", meter)?;
    let [
        role,
        name,
        version,
        kind,
        repository,
        reference_kind,
        reference,
        precision,
        revision,
    ] = parts.as_slice()
    else {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded source-v1 descriptor requires 9 fields",
        ));
    };
    if !matches!(
        role.as_ref(),
        "configured-locked" | "structural-observation"
    ) || name.is_empty()
        || !meter.compare(name, package)?.is_eq()
        || version.is_empty()
        || repository.is_empty()
        || !matches!(
            kind.as_ref(),
            "git" | "registry" | "sparse" | "path" | "local-registry" | "directory"
        )
    {
        return Err(Failure::report(
            meter,
            E::Value,
            &"invalid embedded source descriptor",
        ));
    }
    validate_repository(repository, kind.as_ref() == "git", meter)?;
    let reference_valid = match reference_kind.as_ref() {
        "absent" | "default-branch" => reference.is_empty(),
        "branch" | "tag" | "revision" => !reference.is_empty(),
        _ => false,
    };
    meter.work(revision.len())?;
    let revision_valid = match precision.as_ref() {
        "absent" => revision.is_empty(),
        "present" => {
            let mut decoded = [0; 32];
            matches!(revision.len(), 40 | 64)
                && purrdf_hash::hex::decode_canonical_to_slice(revision, &mut decoded).is_ok()
        }
        _ => false,
    };
    if !reference_valid
        || !revision_valid
        || kind.as_ref() == "git" && reference_kind.as_ref() == "absent"
        || kind.as_ref() != "git"
            && (reference_kind.as_ref() != "absent" || precision.as_ref() != "absent")
    {
        return Err(Failure::report(
            meter,
            E::Value,
            &"invalid embedded source reference or revision fields",
        ));
    }
    Ok(())
}

fn validate_target<'m>(text: &str, meter: &'m Meter) -> Result<(), Failure<'m>> {
    let parts = components(text, "target-v1", meter)?;
    let [source, manifest, package, kind, name, root, crate_path] = parts.as_slice() else {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded target-v1 descriptor requires 7 fields",
        ));
    };
    validate_source(source, package, meter)?;
    canonical_path(manifest, true, meter)?;
    canonical_path(root, false, meter)?;
    if package.is_empty()
        || name.is_empty()
        || crate_path.is_empty()
        || !matches!(
            kind.as_ref(),
            "lib" | "bin" | "test" | "bench" | "example" | "build-script"
        )
    {
        return Err(Failure::report(
            meter,
            E::Value,
            &"invalid embedded target descriptor",
        ));
    }
    Ok(())
}

fn validate_optional<'m>(text: &str, target: bool, meter: &'m Meter) -> Result<bool, Failure<'m>> {
    if meter.compare(text, "#absent")?.is_eq() {
        return Ok(false);
    }
    let parts = components(text, "present", meter)?;
    let [value] = parts.as_slice() else {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded present descriptor requires 1 field",
        ));
    };
    if target {
        validate_target(value, meter)?;
    } else if value.is_empty() {
        return Err(Failure::report(
            meter,
            E::Value,
            &"empty present embedded identity",
        ));
    }
    Ok(true)
}

fn validate_trait<'m>(text: &str, meter: &'m Meter) -> Result<(), Failure<'m>> {
    if meter.compare(text, "#absent")?.is_eq() {
        return Ok(());
    }
    if text.starts_with("#canonical/") {
        let parts = components(text, "canonical", meter)?;
        let [name] = parts.as_slice() else {
            return Err(Failure::report(
                meter,
                E::Value,
                &"embedded canonical descriptor requires 1 field",
            ));
        };
        if name.is_empty() {
            return Err(Failure::report(
                meter,
                E::Value,
                &"empty canonical embedded trait key",
            ));
        }
    } else {
        let parts = components(text, "unresolved", meter)?;
        if parts.len() < 2 {
            return Err(Failure::report(
                meter,
                E::Value,
                &"invalid unresolved embedded trait candidates",
            ));
        }
        for part in parts.iter() {
            meter.work(1)?;
            if part.is_empty() {
                return Err(Failure::report(
                    meter,
                    E::Value,
                    &"invalid unresolved embedded trait candidates",
                ));
            }
        }
        for paths in parts[1..].windows(2) {
            if !meter.compare(&paths[0], &paths[1])?.is_lt() {
                return Err(Failure::report(
                    meter,
                    E::Value,
                    &"invalid unresolved embedded trait candidates",
                ));
            }
        }
    }
    Ok(())
}

fn validate_owner<'m>(text: &str, meter: &'m Meter) -> Result<(), Failure<'m>> {
    let parts = components(text, "owner-v1", meter)?;
    let [
        target,
        kind,
        symbol,
        self_type,
        trait_path,
        trait_key,
        definition,
    ] = parts.as_slice()
    else {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded owner-v1 descriptor requires 7 fields",
        ));
    };
    validate_target(target, meter)?;
    let has_self = validate_optional(self_type, false, meter)?;
    let has_trait = validate_optional(trait_path, false, meter)?;
    validate_trait(trait_key, meter)?;
    validate_optional(definition, true, meter)?;
    let shape = match kind.as_ref() {
        "function" | "trait-default-method" => !has_self && !has_trait,
        "inherent-method" => has_self && !has_trait,
        "trait-impl-method" => has_self && has_trait,
        _ => false,
    };
    if symbol.is_empty() || !shape {
        return Err(Failure::report(
            meter,
            E::Value,
            &"invalid embedded owner descriptor",
        ));
    }
    Ok(())
}
