## Request

Provide a public `purrdf-toml` profile and codec analogous to `purrdf-markdown` and `purrdf-json`: TOML remains a convenient human authoring format while its structure and exact source bytes have a queryable RDF 1.2 representation.

## Concrete consumer

RAF's Census ledger needs strict configuration for jobs, preferred providers, variants and sanctioned duplicate sets. We want to keep format parsing, source spans, identities, RDF projection and reconstruction in PurRDF, with only Census-specific schema validation in RAF. See [RAF ledger work](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/4) and [PurRDF reuse priority](https://github.com/Blackcat-Informatics/rust-agentic-foundations/issues/53).

Current inspected PurRDF main has reusable Markdown and JSON profiles, but the TOML reader at `crates/helper-census/src/toml.rs` is private to an unpublished binary. A reusable TOML profile would prevent another project-local format implementation.

## Requested contract

- A public Rust API with an explicit, versioned profile, vocabulary and resource bounds, following established analyze/model/project/decode conventions where appropriate. State the supported TOML specification version.
- A typed model for tables, arrays of tables, inline tables, dotted and quoted keys, arrays, strings, integers, floats and TOML date/time values. Preserve occurrence identity, source order, parentage and exact byte spans; distinguish explicit and implicit table structure where needed to reconstruct declarations.
- Lossless source preservation, including comments, whitespace, line endings, quote and escape choices, numeric spelling and declaration order. Semantic values and verbatim source must remain distinguishable.
- Projection through existing native RDF 1.2 infrastructure, reusing shared byte-cover, identity and reconstruction machinery rather than a second graph or codec kernel. Explicit source/document selection when decoding.
- Decode must reconstruct exact bytes and verify asserted structure and identities against the reparsed document, as the JSON codec does. Missing, conflicting or forged cover/structure data must fail rather than produce partial success.
- Actionable parse and resource errors with source locations. Bound input size, nesting, node counts and retained path/span work without native-stack overflow. Preserve TOML distinctions such as date/time kinds and non-finite float values without silently coercing them to strings or JSON.
- Keep application policies separate: Census owns required keys, unknown-key refusal, provider rules and extension validation. The TOML profile should accept valid TOML without embedding the Census ledger schema or legacy dialect rules.

## Acceptance

An example TOML ledger can be analyzed, projected to RDF, queried for a job and its fields, and decoded byte-for-byte. Fixtures cover all TOML value/table forms, comments and CRLF, invalid TOML and duplicate-key/table declarations, bounded refusals with valid neighbours, and graph tampering. The public core should remain usable without filesystem or process access, including WASM.

This is a reusable TOML authoring-to-RDF surface, not a request to convert TOML through JSON or add another private subset parser.
