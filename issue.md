# Issue #374: census: compare branded Cap’n Proto superclasses

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Compare committed Cap’n Proto interface inheritance when either side of
a direct superclass edge has lexical type parameters.

- A superclass edge retains its target ID and canonical brand. A branded
  superclass requires complete bindings for all parameter-bearing scopes
  in its declaration chain. Bindings may use the derived interface's
  proved lexical parameters. A nongeneric superclass requires an empty
  brand, including when the derived interface is generic.
- Keep reciprocal declaration ownership and consistent generic flags.
  A compiler request may list the same superclass ID with distinct
  complete brands. Keep each instantiation and refuse a repeated
  canonical edge, a cycle or a malformed brand.
- Keep valid additions compatible. Report a removed superclass or a
  changed brand on the interface that declares the edge. Compare each
  inherited method layout once by its defining interface ID.
- Use compiler-produced concrete and lexical superclass bindings,
  including a generic declaration nested under another generic scope.
  Exercise compatible additions, distinct instantiations of one direct
  superclass, changed bindings and inherited layouts beside controlled
  refusals. A diamond still compares its defining declaration once.

Reuse the type-reading scopes from #371. Parent #214 retains unbound
endpoint forms and the remaining application and compiler profiles.


## Comments (0)

