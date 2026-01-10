### Project Scope & Philosophy

Egor is a **graphics engine**, not a game engine

Changes must:

- Be broadly applicable to **apps, tools and games**
- Avoid assumptions about game-specific concepts (entities, scenes, physics, AI, etc)
- Keep APIs generic and composable

If a change only makes sense for games, it likely does not belong in egor

### Commit Messages

Egor uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/). This helps `git-cliff` parse git history for generating clean release notes

Format:
type(scope): short description

Examples:

- fix(renderer): handle `begin_frame()` returning `None`
- refactor(app): move window title into `AppConfig`
- Avoid vague types/messages (e.g. `chore: stuff`, `fix: misc`)
- docs(readme): clarify wasm setup

Rules:

- One logical change per commit
- No unrelated commits in a single PR
- Squash commits are preferred if history gets messy

### Pull Requests

Before opening a PR:

- [ ] PR addresses **one** issue or feature
- [ ] Commits are clean and logically separated
- [ ] No unrelated refactors or formatting changes
- [ ] Code builds on native **and** wasm (if applicable)
- [ ] Public API changes are explained in the PR description

Scope check:

- Does this help **any 2D app**, or only a game?
- Can this be implemented outside egor by the user?
- Does this reduce flexibility or lock users into a pattern?
- Would this make egor harder to use as a general-purpose graphics library?

If unsure, open an issue or ask in [Discord](https://opensourceforce.net/discord) first

Large or architectural changes should be discussed in an issue first

PRs that mix unrelated changes may be closed or asked to split

### Pre-PR Checklist

Before submitting, make sure your changes pass these commands:

```bash
# Check formatting
cargo fmt --all -- --check

# Build and test native
cargo test --all-targets --all-features --locked

# Lint with Clippy
cargo clippy --all-targets --all-features --locked -- -D warnings -A clippy::new-without-default

# Verify documentation builds
cargo doc --no-deps --document-private-items --locked
```
