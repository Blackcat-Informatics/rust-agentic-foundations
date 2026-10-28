// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Private native object writes with the observed Git read authorities.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};

use super::{query, text};
use crate::exec::{ExecError, Program, Runner, ViewError};

use super::super::git_context::OBJECT_READS;

pub(super) fn observe(runner: &dyn Runner, root: &Path) -> Result<Vec<PathBuf>, ExecError> {
    let primary = query(
        runner,
        root,
        Program::Git,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "objects",
        ],
    )?;
    let primary = text(primary, "read effective Git object directory")?;
    let primary = primary.strip_suffix('\n').ok_or_else(|| {
        ViewError::refusal(
            "read effective Git object directory",
            "missing final newline",
        )
    })?;
    let mut paths = vec![
        std::fs::canonicalize(primary)
            .map_err(|e| ViewError::new("resolve effective Git object directory", e))?,
    ];
    let alternates = query(runner, root, Program::Git, &["count-objects", "-v"])?;
    let alternates = text(alternates, "read native Git object alternates")?;
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut known = OrderedTable::default();
    known
        .insert(paths[0].clone(), (), &mut memory)
        .map_err(|e| ViewError::new("index Git object authority", e))?;
    for line in alternates.lines() {
        let Some(path) = line.strip_prefix("alternate: ") else {
            continue;
        };
        let path = std::fs::canonicalize(decode_path(path)?)
            .map_err(|e| ViewError::new("resolve native Git object alternate", e))?;
        if known
            .insert(path.clone(), (), &mut memory)
            .map_err(|e| ViewError::new("index Git object authority", e))?
            .is_none()
        {
            if paths.len() == super::super::capture::MAX_ENTRIES {
                return Err(ViewError::refusal(
                    "retain Git object authority",
                    "directory count limit exceeded; reduce Git object alternates",
                )
                .into());
            }
            memory
                .push(&mut paths, path)
                .map_err(|e| ViewError::new("retain Git object authority", e))?;
        }
    }
    Ok(paths)
}

pub(super) fn create(path: &Path, sources: &[PathBuf]) -> Result<(), ExecError> {
    super::create_private(path)
        .map_err(|e| ViewError::new("reserve private Git object store", e))?;
    let info = path.join("info");
    super::create_private(&info)
        .map_err(|e| ViewError::new("reserve private Git object metadata", e))?;
    let reads = path.join(OBJECT_READS);
    super::create_private(&reads)
        .map_err(|e| ViewError::new("reserve Git object read aliases", e))?;
    let mut alternates = String::new();
    for (ordinal, source) in sources.iter().enumerate() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(source, reads.join(ordinal.to_string()))
            .map_err(|e| ViewError::new("link observed Git object authority", e))?;
        #[cfg(not(unix))]
        return Err(ViewError::refusal(
            "link observed Git object authority",
            "native completed views require Unix lexical links",
        )
        .into());
        writeln!(alternates, "{OBJECT_READS}/{ordinal}").expect("writing to String cannot fail");
    }
    std::fs::write(info.join("alternates"), alternates)
        .map_err(|e| ViewError::new("write private Git object read authorities", e))?;
    Ok(())
}

/// Git's C-quoted pathname transport, not RDF or document string syntax.
/// Native Git enumerates the effective alternate chain; only its path carrier
/// is decoded here. Unquoted paths and octal byte escapes retain native bytes.
fn decode_path(text: &str) -> Result<PathBuf, ExecError> {
    let Some(quoted) = text.strip_prefix('"') else {
        return Ok(PathBuf::from(text));
    };
    let quoted = quoted.strip_suffix('"').ok_or_else(|| {
        ViewError::refusal("decode Git object alternate", "unterminated quoted path")
    })?;
    let mut bytes = Vec::new();
    let mut input = quoted.bytes();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    while let Some(byte) = input.next() {
        let byte = if byte == b'\\' {
            let escaped = input.next().ok_or_else(|| {
                ViewError::refusal("decode Git object alternate", "unfinished path escape")
            })?;
            match escaped {
                b'"' | b'\\' => escaped,
                b'a' => 7,
                b'b' => 8,
                b't' => b'\t',
                b'n' => b'\n',
                b'v' => 11,
                b'f' => 12,
                b'r' => b'\r',
                b'0'..=b'3' => {
                    let mut value = escaped - b'0';
                    for _ in 0..2 {
                        let digit = input
                            .next()
                            .filter(|byte| matches!(byte, b'0'..=b'7'))
                            .ok_or_else(|| {
                                ViewError::refusal(
                                    "decode Git object alternate",
                                    "invalid octal path escape",
                                )
                            })?;
                        value = value * 8 + (digit - b'0');
                    }
                    value
                }
                _ => {
                    return Err(ViewError::refusal(
                        "decode Git object alternate",
                        "unknown path escape",
                    )
                    .into());
                }
            }
        } else {
            byte
        };
        if byte == 0 {
            return Err(
                ViewError::refusal("decode Git object alternate", "NUL in native path").into(),
            );
        }
        memory
            .push(&mut bytes, byte)
            .map_err(|e| ViewError::new("retain native Git alternate path", e))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        Ok(PathBuf::from(std::ffi::OsString::from_vec(bytes)))
    }
    #[cfg(not(unix))]
    Ok(PathBuf::from(String::from_utf8(bytes).map_err(|e| {
        ViewError::new("decode native Git alternate path", e)
    })?))
}
