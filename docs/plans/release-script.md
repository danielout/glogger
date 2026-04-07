# Release Process Improvements

## Current State

The release process is already fairly automated:

- **Trigger:** Manual `workflow_dispatch` in GitHub Actions with a version bump type (patch/minor/major/explicit)
- **Prepare job:** Runs `scripts/bump-version.sh` (updates version across 5 files), generates a changelog from commit prefixes, commits + tags + pushes
- **Build job:** Parallel builds on Windows, macOS, and Linux. Collects MSI, NSIS .exe, DMG, .deb, and AppImage
- **Publish job:** Creates a GitHub Release on `danielout/glogger-release` with all artifacts attached
- **Update notifications:** The app already checks the GitHub Releases API on startup and hourly, shows a toast + bottom bar link when a new version is found (see `docs/features/update-notifications.md`)

## Improvements

### 1. User-Friendly Download Labels

**Problem:** GitHub release assets have raw filenames like `glogger_0.4.5_x64_en-US.msi` and `glogger_0.4.5_amd64.AppImage`. Non-technical users visiting the release page can't easily tell which file is for their OS.

**Solution:** Add a release body section with clear download links and labels. The changelog script or publish step should generate a "Downloads" section at the top of the release notes:

```markdown
## Downloads

| Platform | Installer |
|----------|-----------|
| **Windows** (recommended) | [glogger-0.5.0-setup.exe](link) |
| **Windows** (MSI) | [glogger-0.5.0.msi](link) |
| **macOS** | [glogger-0.5.0.dmg](link) |
| **Linux** (Debian/Ubuntu) | [glogger-0.5.0.deb](link) |
| **Linux** (AppImage) | [glogger-0.5.0.AppImage](link) |
```

This can be done in the publish step after artifact upload -- query the release assets via `gh api` and rebuild the release body with the download table prepended to the changelog.

### 2. Rename Artifacts for Clarity

The Tauri build output uses its own naming conventions. Add a rename step after collecting artifacts so filenames are more intuitive:

- `glogger_0.5.0_x64-setup.exe` -> `glogger-0.5.0-windows-setup.exe`
- `glogger_0.5.0_x64_en-US.msi` -> `glogger-0.5.0-windows.msi`
- `glogger_0.5.0_amd64.deb` -> `glogger-0.5.0-linux.deb`
- `glogger_0.5.0_amd64.AppImage` -> `glogger-0.5.0-linux.AppImage`
- `glogger_0.5.0_x64.dmg` -> `glogger-0.5.0-macos.dmg`

### 3. Conventional Commit Enforcement

The changelog generator already categorizes by prefix (`feat:`, `fix:`, etc.), but nothing enforces developers actually use them. Most recent commits are unprefixed ("bad text on npc browser", "gourmand screen scroll fix") so they all land in "Other."

Options (pick one):
- **Lightweight:** Add a commit-msg git hook (via a setup script or `.githooks/`) that warns when no known prefix is used but doesn't block
- **Stricter:** Block commits that don't match a known prefix pattern. Probably overkill for a solo/small-team project right now

### 4. Pre-Release Validation

Add a check step before the version bump to catch obvious issues:
- `npm run build` (frontend compiles)
- `cargo check` (Rust compiles)
- Run any existing tests (`cargo test`, `npm test` if applicable)

This prevents cutting a release tag that won't even build. Can be a separate `validate` job that `prepare` depends on, or just inline steps.

### 5. Drop "alpha" from Window Title at Beta

`tauri.release.conf.json` currently sets the title to "glogger alpha v0.4.5". When we hit beta, `bump-version.sh` should be updated (or a separate flag added) to change "alpha" to "beta" in the window title. Not urgent but worth remembering.

### 6. Release Candidate / Draft Workflow

When nearing beta, consider adding a `--draft` flag to the workflow so releases can be created as drafts, reviewed, then manually published. This is a one-line change to the `gh release create` command (`--draft`).

## Out of Scope (For Now)

- **Auto-update (Tauri updater):** Too much complexity for the current stage -- code signing, update server, delta patches. The current "go download it" notification works fine.
- **Code signing / notarization:** Will matter for beta/GA but not worth the setup cost yet.
- **Multiple architecture builds:** Currently x64 only. ARM/Apple Silicon can wait until there's demand.

## Status

- [x] Rename artifacts + download table in release notes
- [x] Pre-release validation step in CI
- [x] Conventional commit hook (warning-only)
- [ ] Draft releases (useful closer to beta)
- [ ] Window title "alpha" -> "beta" update (do it when we hit beta)
