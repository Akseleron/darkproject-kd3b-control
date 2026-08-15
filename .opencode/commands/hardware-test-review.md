---
description: Review a proposed real-keyboard write test before any hardware command is executed
---

Read @docs/SAFETY.md and the protocol document for the proposed operation.

Do not execute the test.

Return a preflight report containing:

- exact VID/PID;
- exact transport/interface selection;
- operation being tested;
- exact packet bytes or deterministic encoder path;
- evidence supporting every written field;
- whether the write is expected to be volatile or persistent;
- expected visible result;
- recovery/rollback behavior;
- tests that already prove packet construction;
- any reason the test should be blocked.

If any byte or transport assumption is undocumented, block the test.
