# bgit hooks and execution order

How hooks work in bgit, where to place them, and the order they run.

## Quick rules

- Put portable, repo-scoped hooks under `.bgit/hooks/` (checked in, cross‑platform).
- Naming: `pre_<event>` and `post_<event>` (e.g., `pre_git_commit`, `post_git_commit`).
- Standard Git hooks supported: commits only — `pre-commit` and `post-commit`.
- Other client‑side native hooks (e.g., `pre-push`, `commit-msg`) are not run; a warning is logged at workflow start if they’re detected.

## Hook locations and naming

- Portable hooks: `<repo>/.bgit/hooks/`
  - File names follow bgit event names: `pre_<event>` and `post_<event>`.

- Native hooks: resolved from `git config core.hooksPath` (local → global) or `<repo>/.git/hooks/`.
  - Only `pre-commit` and `post-commit` are executed by bgit.

Examples:

- Commit: `.bgit/hooks/pre_git_commit`, `.bgit/hooks/post_git_commit`
- Other events: `.bgit/hooks/pre_git_add`, `.bgit/hooks/post_git_add`, etc.

## Execution order for a commit

1. `.bgit/hooks/pre_git_commit` (if present)
2. Standard Git `pre-commit` (if present)
3. Commit action
4. `.bgit/hooks/post_git_commit` (if present)
5. Standard Git `post-commit` (if present)

## Platform notes

- Unix: bgit ensures `.bgit/hooks/*` are executable (+x) before running.
- Windows: for `.bgit/hooks`, common extensions are supported (`.bat`, `.cmd`, `.ps1`, `.exe`, or no extension) and executed with an appropriate host.

## Logging and diagnostics

- Hook stdout/stderr are captured and logged; adjust verbosity with `-v` flags.
- At workflow start, bgit logs the resolved native hooks path and any non‑sample hooks found there, and warns about unsupported native hooks.

## Tips

- Prefer `.bgit/hooks` for team‑friendly, cross‑platform behavior.
- Keep native hooks minimal since only commit hooks are executed by bgit.
- If you rely on native `commit-msg`/`pre-push`, migrate the logic to appropriate `.bgit/hooks` events.

## FAQ

- Why not run all native Git hooks?
  - libgit2 (used by bgit) doesn’t invoke native hooks. bgit bridges commit hooks for compatibility and uses `.bgit/hooks` for portability and predictability elsewhere.
