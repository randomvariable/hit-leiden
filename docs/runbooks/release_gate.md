# Release Gate Runbook

- Use pinned hardware profile.
- Compare against frozen baseline commit.
- Reject release-gate if source uses live per-step queries.
- Emit reason code in benchmark outcome when ineligible.
