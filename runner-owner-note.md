The registered Census check in PR #66 now calls the same shared
`raf_workplan::exec::SystemRunner` through `cargo_census::registered::run`.
Please include that production caller in this issue's existing containment and
parent-death acceptance scope, alongside Workplan evidence execution.

The shared runner starts a separate Unix process group and enforces timeout and
output cleanup while its host is alive. It does not forward terminal interruption
or establish child cleanup after host death; a detached descendant can also
escape the existing group kill. The current implementation and its existing
documented limitation are unchanged by PR #66. Its registered-check contract
claims the existing runner's execution/output limits and trusted-code boundary,
not cancellation containment. Timeout/overflow tests are not cancellation proof.

This issue retains the actual Linux/Windows containment, unsupported-host hard
refusal, parent-death behavior and counted-clock requirements. Neither the new
registered caller nor its successful check protocol closes those requirements.
