// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Windows: the path the system gives for an open handle. This is the crate's
//! only call into a foreign function.

use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::windows::ffi::OsStringExt as _;
use std::os::windows::io::AsRawHandle as _;
use std::path::PathBuf;

use windows_sys::Win32::Storage::FileSystem::{
    FILE_NAME_NORMALIZED, GetFinalPathNameByHandleW, VOLUME_NAME_DOS,
};

/// The normalised path of `file`, with its drive letter, in the verbatim
/// (`\\?\`) form that `std::fs::canonicalize` also gives.
#[allow(
    unsafe_code,
    reason = "GetFinalPathNameByHandleW is a foreign function, and this is the crate's one call into one"
)]
pub(super) fn of(file: &File) -> io::Result<PathBuf> {
    let mut buffer = vec![0_u16; 512];
    loop {
        let capacity = u32::try_from(buffer.len()).map_err(io::Error::other)?;
        // SAFETY: `file` borrows the `File` that owns the handle, so the
        // handle stays open for the whole call. `buffer` is a live, writable
        // allocation of exactly `capacity` UTF-16 units, and the function
        // writes at most `capacity` units into it.
        let written = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                buffer.as_mut_ptr(),
                capacity,
                FILE_NAME_NORMALIZED | VOLUME_NAME_DOS,
            )
        };
        let written = usize::try_from(written).map_err(io::Error::other)?;
        if written == 0 {
            return Err(io::Error::last_os_error());
        }
        if written < buffer.len() {
            // The path fitted: `written` units, without the terminating NUL.
            buffer.truncate(written);
            return Ok(PathBuf::from(OsString::from_wide(&buffer)));
        }
        // The buffer was too small: `written` is the size the path needs,
        // terminating NUL included. It grows strictly, so the loop ends even
        // if the call ever answered with the size it was given.
        buffer.resize(written.max(buffer.len() + 1), 0);
    }
}
