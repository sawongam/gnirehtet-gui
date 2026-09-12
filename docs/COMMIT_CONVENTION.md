# Commit convention

**Policy:** [Conventional Commits](https://www.conventionalcommits.org/) — **strict**.

## Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

- **Scope is optional** (not required).
- Description: imperative mood, lowercase after the type, no trailing period, ~≤72 chars preferred.
- One logical change per commit when practical.

## Allowed types

| Type | Use for |
|------|---------|
| `feat` | New user-facing capability |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `chore` | Tooling, repo hygiene, non-product noise |
| `refactor` | Code change with no feature/fix intent |
| `test` | Tests only |
| `perf` | Performance |
| `ci` | CI config |
| `build` | Build system / dependencies |
| `style` | Formatting only (no logic change) |
| `revert` | Revert a prior commit |

## Examples

```
feat: add poll_owned_relay for crash watcher
fix: align Phase 0 orchestrator with ERROR_UX acceptance
docs: add official TEAM_BRIEF for MVP scope
chore: align branch names to master and dev
```

## Not allowed

Freeform subjects (`Add X`, `Wire Y`, `Finish Z`) without a conventional type prefix.

## Branches

- `master` — baseline
- `dev` — active development

Author identity for this repo: `sawongam <sangamadhikari.61@gmail.com>`.
