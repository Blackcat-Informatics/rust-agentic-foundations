**What I am least confident about, and why**
* **The normalisation choices now frozen into normal form 1.** Thresholds count structural tokens before erasures. Locals are numbered after the highest parameter number the body uses, placeholders included. Names inside attributes are kept. Integers are respelled to decimal before the literal tier. Each choice was adjudicated by an independent review and is pinned by a golden, but any later change to one of them is a `normal_form` bump plus a migration.
* **Near-miss (Type-3) duplicates.** Bodies with one statement added or reordered are not grouped. This was declined on purpose: its parameters would have to be frozen into v1 without data. `explain(a, b)` makes such misses visible pair by pair.

**What you may not know but should**
* The job-match classification moved from the ledger's change record into this change. The ledger change now only parses `[[job]]` tables into `raf_census::Job` and refuses a mismatched `normal_form`.
* The census reads `raf-source`'s parse. A file `raf-source` cannot load is a load error, never a skipped file.
