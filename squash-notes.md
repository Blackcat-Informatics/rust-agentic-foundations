feat(workplan): accept a review only with a clean verdict at current digests when strict

A capability may state review_strict: true. Under it, a required review is satisfied only by a
completed review of the required kind whose verdict is clean and whose input digests are
current. A review with findings, or a clean review whose inputs have since changed, gets a
review-stale finding that names which condition failed.

The later completed review of the same scope and kind decides, in either direction. Blocked
reviews are ignored. The evidence text for the review obligation now states the clean-verdict
condition, and the reconcile contract table matches it.

A capability that does not state the key is judged exactly as before. The key is optional and
defaults to false. An older binary refuses a document that states the key, because the key set
is closed.

Defect-Class: none
