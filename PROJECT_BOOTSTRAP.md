# Bootstrap checklist

Before the first OpenCode implementation session:

1. Put this repository in its final local directory.
2. Initialize local Git if not already initialized.
3. Run `./scripts/check-env.fish`.
4. Open the repository root in OpenCode.
5. Use `/context`.
6. Use `/plan-sprint`.
7. Review the plan.
8. Paste/use `prompts/FIRST_IMPLEMENTATION_PROMPT.md` with the implementation agent.
9. Keep all first-task work offline/mock-only.
10. Do not perform any keyboard write until the protocol encoder and transport selection have been separately reviewed.

Suggested agent workflow in the shown OpenCode setup:

- Plan validation: `Prometheus - Plan Builder`.
- Implementation: `Sisyphus - Ultraworker`.
- Deep reverse-engineering tasks: use a research/deep agent only for evidence analysis, then have the primary implementation agent make the scoped code change.
- Keep the high reasoning setting for protocol/architecture tasks; lower it only for routine mechanical edits if desired.

Do not let multiple agents simultaneously edit the same protocol file during reverse engineering. Parallelize independent work such as capture analysis, UI research, and test review instead.
