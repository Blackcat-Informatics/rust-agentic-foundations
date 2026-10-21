Root discovery now walks bounded Cargo manifests through `raf-source::Tree`, so it works with `MemoryTree` and finds member roots without launching Cargo. The workplan library and CLI use that implementation; the metadata subprocess, JSON scanner and their limits are removed.

Standalone and excluded packages use their own directory, nested workspaces use the nearest root, and ambiguous membership or malformed manifests fail with actionable errors. Explicit sibling workspaces are supported. Contracts and security boundaries document ancestor-manifest reads and canonical directory requirements.

Validation: full `make check`, both workplan completion gates, the normal commit hook and all 12 hosted jobs passed, including native macOS and Windows tests. MemoryTree/disk fixtures cover zero Runner calls, manifest bounds, unsafe paths and sibling workspace membership.

Positive CLI root fixtures remain. Assertions requiring the removed metadata child were replaced with zero-spawn checks.

Closes #42.
