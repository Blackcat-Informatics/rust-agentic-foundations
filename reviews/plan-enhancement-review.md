# Enhancement / utility review (independent, single concern) — condensed record

Run read-only during plan mode; full report returned inline. VERDICT: NEEDS-REVISION (15 must-have, 8 optional).

Must-have: 1 closure parse entry plus `pub use syn`; 2 item spans, body spans, (file, offset) identity; 3 Workspace::source; 4 file table and file->items lookup; 5 item->target, Cargo names, [lib] name, bin crate path; 6 documented / doc_hidden; 7 Resolution with via_glob, Ambiguous carries candidates; 8 items(), item(id), packages(); 9 package without [workspace] is a discovery leaf; 10 all-level headings and anchors; 11 prose view and placeholder rule in the profile; 12 generated/PROPOSED full block shapes; 13 one splice path covering index.md; 14 value constructors; 15 section emptiness ignores generated blocks.
Optional: 16 validate returns typed document (adopted); 17 link target classification (adopted); 18 finer item kinds and test roles (adopted); 19 shared Span crate (declined); 20 include! following (declined); 21 reverse references / effective visibility (declined); 22 split links (user decided: refuse); 23 DiskTree for docs_contract (declined).

Dispositions: plan.md section "F. Review dispositions".
