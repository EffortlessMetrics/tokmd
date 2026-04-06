# Shared Agent Roles

These role documents define the canonical responsibilities for checked-in agent adapters in this repository.

Each adapter-specific manifest under `.claude/agents/` or `.jules/agents/` should point back to one of these shared roles instead of re-inventing policy locally.

## Active scheduled agents

| Agent | Emoji | Role doc | Focus |
|-------|-------|----------|-------|
| Bolt | ⚡ | [bolt.md](bolt.md) | Performance |
| Auditor | 🧾 | [auditor.md](auditor.md) | Dependency hygiene |
| Gatekeeper | 🧪 | [gatekeeper.md](gatekeeper.md) | Quality / determinism |
| Librarian | 📚 | [librarian.md](librarian.md) | Docs / examples |
| Palette | 🎨 | [palette.md](palette.md) | UX / developer experience |
| Compat | 🧷 | [compat.md](compat.md) | Feature/matrix compatibility |

## Supporting roles

| Role | Role doc | Focus |
|------|----------|-------|
| Author | [author.md](author.md) | Implement one narrow change cleanly |
| CI | [ci.md](ci.md) | CI reliability |
| Critic | [critic.md](critic.md) | Adversarial review |
