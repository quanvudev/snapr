# Contributing to Snapr

Thanks for your interest in Snapr! Bug reports, ideas, feedback and code are all welcome.

## Feedback and ideas

You don't need to write code to help. Open an [issue](https://github.com/quanvudev/snapr/issues/new/choose) and pick a template:

- **Bug report:** something doesn't work as expected. Include your Windows version, the Snapr version (Settings → About) and the steps to reproduce it.
- **Feature request:** an idea for something new or a better way to do something. Describe the problem you want solved, not only the solution.
- **Feedback:** anything else: what you like, what feels awkward, what confused you.

Please search the existing issues first and add a 👍 or a comment to one that already covers your point.

## Code contributions

All changes go through a pull request to `master`; nobody pushes to it directly.

1. **Talk first for anything big.** For a new feature or a large change, open an issue (or comment on an existing one) before you start, so we can agree on the approach and you don't spend time on something that won't be merged. Small fixes, typos and translations can go straight to a PR.
2. **Fork** the repository and create a branch from `master` with a short name, e.g. `fix/toast-dpi` or `feat/blur-tool`.
3. **Make your change**, following the conventions below.
4. **Check it** (see [Checks](#checks)) and test it by hand in `bun run tauri dev`.
5. **Open a pull request** against `master` and fill in the template. Keep one change per PR; a small PR is reviewed much faster than a large one.

A maintainer reviews every PR. Expect some back-and-forth; it's normal and not a judgment of your work.

### Setup

You need Windows 10 or 11, [Bun](https://bun.sh), [Rust](https://rustup.rs) (stable, MSVC) and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```powershell
bun install
bun run tauri dev
```

[CLAUDE.md](CLAUDE.md) describes the architecture: the two event loops, every backend module and how the settings UI talks to the backend. Reading the parts you touch will save you a lot of time.

### Conventions

- Use **bun**, never npm, pnpm or yarn.
- **No code comments.** Prefer clear names and small functions.
- **UI** is built from the shadcn components in `src/components/ui/`. Don't hand-roll native `<input>` / `<button>` elements; add a missing component with `bunx --bun shadcn@latest add <component>`.
- **Keep it minimal:** reuse existing code, no speculative abstractions, no unrelated refactors in the same PR.
- **Text** shown to users goes into `src/lib/i18n.ts` in both English and Vietnamese. If you don't speak Vietnamese, add the English text and say so in the PR; we'll translate it.
- **Config:** a new field in `config.rs` needs a `#[serde(default)]` (or a default function) so existing `config.toml` files keep loading.
- **Commits** follow [Conventional Commits](https://www.conventionalcommits.org): `type(scope): Summary`, e.g. `fix(toast): Keep the preview on the cursor's monitor` or `feat(draw): Add a blur tool`. Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `chore`.
- **Don't** bump versions or edit `CHANGELOG.md`; that happens at release time.

### Checks

There is no automated test suite, so these and a manual test are what keep `master` healthy:

```powershell
bunx tsc --noEmit                 # frontend type check
cd src-tauri; cargo clippy        # backend lint, no new warnings
```

In the PR, say what you tested by hand (which captures, which settings, single or multiple monitors, DPI scaling).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
