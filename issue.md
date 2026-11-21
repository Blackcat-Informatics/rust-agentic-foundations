# Issue #390: Census: compare unbound Cap’n Proto RPC endpoint arguments

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Extend strict Cap'n Proto history from #355, #359 and #371 to named RPC
parameter and result structs that leave type arguments unbound. Preserve
the committed descriptor input and the separate signed admission verdicts
in #214.

An omitted parameter position, an unbound descriptor entry and an explicit
unconstrained `AnyPointer` denote the same argument. Apply that comparison
to the endpoint and enclosing declaration scopes and to nested type
arguments. Keep positional distinctions when other arguments are bound.
Compare each method role independently and each named layout once by its
defining ID. Keep the enclosing parameters in the declaration comparison
so an unbound brand still exposes a changed enclosing parameter count.

- Compiler descriptors for equivalent omitted and explicit arguments pass
  strict history together.
- Changing a bound argument to an unbound argument reports a type change on
  the affected method role.
- A changed enclosing parameter count reports one finding on the shared
  named endpoint declaration, including when all its arguments stay unbound.
- Malformed brands still refuse, including duplicate scopes, zero-parameter
  scope rows, invalid parameter references and non-pointer bindings.
- Existing inline owner and direct superclass admission rules continue to
  apply.


## Comments (0)

