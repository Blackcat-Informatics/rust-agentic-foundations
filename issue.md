# Issue #166: Parse an explicit source corpus on one bounded worker

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Source-policy consumers need to retain a complete parsed corpus for cross-file and multi-pass checks. Calling parse_source repeatedly cannot do this: each callback borrows a non-Send syntax tree that is dropped with its worker. lillith_decodes needs one analysis for stderr and seam verification without running duplicate detection or changing its production dependency graph.

Add a batch parser accepting explicitly selected paths and source texts, existing parser options and a callback. Apply the native per-file safeguards and aggregate accounting, parse the corpus on one bounded worker, and give the callback ownership of the syntax trees. Only Send report data may leave the worker. Reuse the existing parser, typed failures and panic propagation. Do not add source discovery, duplicate policy, a persistent cache or a new orchestration framework.

Acceptance: a real source-analysis consumer performs cross-file checks over the owned trees; valid and invalid inputs exercise the same native limits, diagnostics and callback lifetime; parser failures never produce partial successful reports. Cover the existing single-file caller behavior and affected Census consumers. Update the capability contract and its bindings, run required repository gates, and deliver through stagectl and ghprsq. The Decodes adoption will pin the merged revision and retain its own source-selection and policy semantics.


## Comments (0)

