<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Namespace repair candidate

`../namespace-repair.patch` applies cleanly to local commit 1841a757. Product
source remains unchanged while make check session 44667 owns it. This candidate
has been rustfmt-parsed but is not yet compiled or Gate-tested. Do not describe
it as qualified. `compiler-witness.rs` compiled with edition 2024, metadata only,
exit 0; receipt: `../raw/namespace-witness-rustc.log`.

The patch retains every written Alias (target, absolute flag and import byte)
under a spelling. Hasher selects the last observation exactly as before; no
detector policy is changed. Qualified heads use Type; bare Value/Macro paths
use their own namespace. Both local shadow stopping and expansion receive the
same native eligibility callback at the actual owner/byte. An alias whose
requested namespace is absent but another indexed namespace exists is ignored
for that lookup. Entirely unindexed external targets keep the existing syntax
observation, explicitly unverified and unusable as provider identity.

The new private signatures are:

- `local(path, namespace, limits, &mut eligible) -> Result<bool, Error>`
- `expand(path, absolute, namespace, limits, &mut eligible) -> Result<Option<Expanded>, Error>`
- `eligible: FnMut(&Alias, Namespace) -> Result<bool, Error>`
- shared `paths::import_eligible(code, owner, byte, alias, namespace) -> Result<bool, CodeResolveError>`

`Expanded::shadowed && Expanded::block` must stop raw lookup: a real block
Type/module binding cannot be replaced by the module's same-spelled target.
Module shadows still permit authoritative raw Source lookup but never spelling
fallback. Own import observations do not rewrite their own target; expansion
also excludes the latest binding while reading its import target, retaining
cycle refusal for a genuine longer alias loop.

Four new functional tests pair qualified value/macro hits with bare shadows,
dependency heads with genuine Type aliases, both namespaces of legal same-head
imports, and competing same-namespace import refusal with a valid disjoint pair.
Native docs sync, all seven Pattern assessment refreshes, focused runtime/lint
and normal gates remain required after application. Source public API and the
provider unit's completed Type-module root correction are deliberately absent.
