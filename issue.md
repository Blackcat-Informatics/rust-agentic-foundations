# Issue #380: Census: initialize a minimal canonical ledger in a fresh workspace

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Give a fresh Rust repository a canonical empty Census policy through `cargo census init`. Discover the workspace root in the same way as `check`, serialize the minimal ledger through the existing RDF writer, and create `helpers-ledger.ttl` exclusively. Preserve any existing policy and debt, including on retries or a concurrent initialization.

Exercise initialization followed by `check` on a clean crate and a planted duplicate. The duplicate must still fail. Document the command, failure on an existing ledger, and the separate explicit debt-adoption operation in the consumer guide.


## Comments (0)

