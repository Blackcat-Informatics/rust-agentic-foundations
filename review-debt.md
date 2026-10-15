# Review debt

PR 18 has no review submissions, no inline comments, no review threads and no bot feedback
(GraphQL: 0 reviews, 0 threads; the 2 issue comments are the author's plan and notes).
Nothing is open, addressed-by-reply or declined.

Required checks: all 12 hosted jobs pass on head d46f95e (fmt, clippy, doc, deny, reuse, msrv,
test, test-release, test-macos, test-windows, cross-check, fuzz). A first run on head a72c5f5
failed test-windows (3 of 19 `bound` tests: shared target directory collisions between
same-named fixtures); fixed in d46f95e with a per-fixture target directory and re-run green.
