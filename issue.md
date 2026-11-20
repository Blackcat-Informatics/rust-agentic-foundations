# Issue #327: census: compare named Cap’n Proto RPC endpoints

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: raf-census

## Body

## Capability

Compare declared nongeneric struct endpoints of selected Cap’n Proto RPC
methods. An interface may use named endpoints, inline parameter or result
lists, or both, and several methods may share the same named struct.

## Acceptance

- Prove each named endpoint is a declared nongroup struct with reciprocal
  lexical ownership and nongeneric ancestry. Refuse corrupt scope and generic
  ancestry. Keep the existing single-owner rules for detached inline wrappers.
- Report a changed endpoint ID or a switch between named and inline forms at
  its method role. Project each retained named layout once through the usual
  type closure, even when it serves several roles or is explicitly selected.
- Use ordinary struct evolution for a named input. Accept a new optional field
  with its implicit compiled default. Keep semantic default and presence
  findings, and retain the explicit-default rule for appended inline inputs.
- Compare controlled compiler requests with shared and mixed endpoint forms.
  Verify the compatible named addition, one layout finding for a shared type,
  changed endpoint identity and malformed scope refusals. Persisted data still
  follows its existing declared struct and enum closure.

Parent #214 retains interface inheritance, generic methods and endpoints,
migration output admission, declaration payload contracts and the remaining
application and compiler profiles.

[Cap’n Proto evolution rules](https://capnproto.org/language.html#evolving-your-protocol)


## Comments (0)

