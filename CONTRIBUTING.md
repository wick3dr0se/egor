### Project Scope & Philosophy

Egor is a **graphics engine**, not a game engine

Changes must:

- Be broadly applicable to **apps, tools and games**
- Avoid assumptions about game-specific concepts (entities, scenes, physics, AI, etc)
- Keep APIs generic and composable

If a change only makes sense for games, it likely does not belong in egor

Scope check:

- Does this help **any 2D app**, or only a game?
- Can this be implemented outside egor by the user?
- Does this reduce flexibility or lock users into a pattern?
- Would this make egor harder to use as a general-purpose graphics library?

### Commit Messages

Egor uses [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/). This helps `git-cliff` parse git history for generating clean release notes

Format:

```
type(scope): short description
```

Rules:

- One logical change per commit
- No unrelated commits in a single PR
- Squash commits are preferred if history gets messy

PRs that mix unrelated changes may be closed or asked to split

### Checks

CI is the reference: see [`.github/workflows/rust.yml`](.github/workflows/rust.yml). A PR has to pass the same commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude xtask --all-targets --features "log ui hot_reload" --locked
cargo doc --workspace --exclude xtask --no-deps --locked
cargo test --workspace --exclude xtask --all-targets --features "log ui hot_reload" --locked
cargo build --target wasm32-unknown-unknown --features "log ui" --locked
```

Miri and the three-OS test matrix run in CI only

If unsure, open an issue or ask in [Discord](https://opensourceforce.net/discord) first

Large or architectural changes should be discussed in an issue first
