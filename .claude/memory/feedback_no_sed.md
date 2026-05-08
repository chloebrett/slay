---
name: No sed commands
description: Never use sed — user must approve each tool call individually and sed bypasses that
type: feedback
---

Never use the `sed` command (including via Bash tool). Use the Edit tool instead — either for targeted replacements or with `replace_all: true` for multiple occurrences.

**Why:** The user must individually approve every tool call. `sed` commands are hard to review and bypass that careful approval workflow.

**How to apply:** Whenever tempted to `sed -i`, use Edit with `replace_all: true` if replacing all instances of a string, or individual Edit calls for targeted changes.
