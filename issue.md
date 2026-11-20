# Issue #322: census: check committed Cap’n Proto file identities

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement

## Body

## Capability

Compare explicitly selected Cap’n Proto file identities across committed
compiler requests. A selected file ID is bound to its canonical requested
filename and matching compiler file node.

Katamari’s schema gate keeps requested filename-to-ID facts alongside
its compiled node facts. Census currently selects data roots and bounded
RPC interfaces. It cannot protect the file’s own requested identity.

## Acceptance

- Select the file ID in the strict history command. Read both descriptors
  from explicit committed revisions. Report removal of the old file ID or
  a changed requested filename for a retained file ID.
- Admit one canonical request row per selected file ID. Require a matching
  file node and display name. Refuse an inconsistent row, a duplicate
  owner or a missing base selection without emitting a partial verdict.
- Allow additional unrelated file requests. Keep persistence declarations
  restricted to declared structs and enums.
- Use controlled compiler descriptors to verify a changed path and an
  admitted neighbor. Verify the inconsistent request and node refusal.

Parent #214 retains history for declarations inside files, migration
output admission, inheritance, named method endpoints, generic RPC and
other application and compiler profiles.


## Comments (0)

