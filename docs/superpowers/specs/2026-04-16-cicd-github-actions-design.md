# CI/CD with GitHub Actions — Design

**Date:** 2026-04-16
**Project:** `murmur` (macOS voice-to-text CLI, Rust)
**Target repo:** `matovu-farid/murmur` (public, to be created)

## Goal

Wire up continuous integration and a fully-automated signed + notarized release flow for the `murmur` CLI, such that tagging `v*.*.*` produces a GitHub Release with macOS arm64 and x86_64 tarballs and an updated Homebrew formula — with zero manual steps between `git push --tags` and users running `brew install`.

## Context

- `murmur` is a single Rust binary (workspace: root `Cargo.toml`, `src/` modules, plus a `lib` crate for tests).
- Tests exist across `notify.rs`, `audio/*`, `transcription/*`, `config/*`, `daemon/*`, `ai/*`, and others.
- Homebrew formula at `Homebrew/murmur.rb` currently has placeholders (`OWNER`, `REPLACE_WITH_*_SHA256`) that the release pipeline will fill on each tag.
- No git remote, no `.github/` directory, no prior CI exist.
- Apple Developer credentials gathered:
  - App Store Connect API Key ID: `ZD9M92A6PY` (`.p8` file at project root, gitignored)
  - Team ID: `9VL7VRY6QZ` (derived from Developer ID cert CN)
  - Developer ID Application cert: installed in login Keychain, valid through Feb 1, 2027
  - Issuer ID: to be obtained by user from App Store Connect API page

## Non-goals

- Linux or Windows builds (macOS-only tool)
- Publishing to crates.io
- Auto-bumping `Cargo.toml` version
- Separate staging / beta channel
- Dependabot or Renovate integration
- Separate `homebrew-tap` repo (deferred; in-repo formula is sufficient for v1)

## Design

### Repository setup

- Create public repo `matovu-farid/murmur` via `gh repo create`.
- Push current `main` (14 commits) as initial history.
- Branch protection on `main`: require CI workflow to pass before merge.
- Replace `OWNER` with `matovu-farid` in `Homebrew/murmur.rb` (lines 3, 8, 13).
- Add `.github/` directory with three workflow files.
- Add `rust-toolchain.toml` pinning `stable` with `rustfmt` + `clippy`.

### Workflow 1: `ci.yml`

**Triggers:** `push` to `main`, `pull_request` to `main`.

**Concurrency:** `group: ci-${{ github.ref }}`, `cancel-in-progress: true`.

**Single job on `macos-14` runner:**

1. Checkout
2. `dtolnay/rust-toolchain@stable` (respects `rust-toolchain.toml`)
3. `Swatinem/rust-cache@v2` (keyed on `Cargo.lock` + target)
4. `cargo fmt --all -- --check`
5. `cargo clippy --all-targets -- -D warnings`
6. `cargo test --all`
7. `cargo build --release`

Expected runtime: 8-10 min cold, 2-3 min cached.

### Workflow 2: `release.yml`

**Trigger:** `push` on tags matching `v*.*.*`.

**Concurrency:** none (releases must complete).

Three sequential jobs:

#### Job A: `build` (matrix)

Runs on `macos-14`. Matrix over `target`:
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

Steps per matrix cell:

1. Checkout
2. Rust toolchain + target via `dtolnay/rust-toolchain@stable` with `targets: ${{ matrix.target }}`
3. Cache via `Swatinem/rust-cache@v2`
4. **Import signing cert into temporary keychain:**
   - `echo "$APPLE_CERTIFICATE_P12_BASE64" | base64 -d > $RUNNER_TEMP/cert.p12`
   - `security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain`
   - `security set-keychain-settings -lut 3600 build.keychain`
   - `security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain`
   - `security import $RUNNER_TEMP/cert.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign`
   - `security list-keychains -d user -s build.keychain login.keychain`
   - `security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" build.keychain`
5. `cargo build --release --target ${{ matrix.target }}`
6. **Sign the binary:** `codesign --sign "Developer ID Application: Farid Matovu ($APPLE_TEAM_ID)" --options runtime --timestamp --force target/${{ matrix.target }}/release/murmur`
7. Verify signature: `codesign --verify --strict --verbose=2 target/${{ matrix.target }}/release/murmur`
8. **Create notarization archive** (notarytool requires `.zip`, `.pkg`, `.dmg` — `.tar.gz` is not accepted):
   - `ditto -c -k --keepParent target/${{ matrix.target }}/release/murmur $RUNNER_TEMP/murmur-notarize.zip`
9. **Notarize:**
   - Write `.p8` key: `echo "$APP_STORE_CONNECT_KEY_P8" | base64 -d > $RUNNER_TEMP/key.p8`
   - `xcrun notarytool submit $RUNNER_TEMP/murmur-notarize.zip --key $RUNNER_TEMP/key.p8 --key-id $APP_STORE_CONNECT_KEY_ID --issuer $APP_STORE_CONNECT_ISSUER_ID --wait`
10. **No stapling.** `xcrun stapler` only supports `.app`, `.pkg`, `.dmg`, `.ipa` bundles — not bare Mach-O executables. Notarization tickets for bare binaries are looked up online by Gatekeeper at first run, based on the binary's signed content hash. This is expected per Apple docs; no extra work needed.
11. **Package for distribution:** tar the signed binary into `murmur-<version>-${{ matrix.target }}.tar.gz` (version = `${{ github.ref_name }}` with `v` prefix stripped). The binary in the tarball is the same one that was notarized, so it inherits the notarization record.
12. Compute SHA256: `shasum -a 256 murmur-<version>-<target>.tar.gz | awk '{print $1}' > <target>.sha256`
13. Upload tarball + sha256 file as artifact named `<target>`.
14. **Cleanup** (always runs via `if: always()`): `security delete-keychain build.keychain` and `rm -f $RUNNER_TEMP/{cert.p12,key.p8,murmur-notarize.zip}`.

#### Job B: `publish` (depends on `build`)

Runs on `macos-14` (or `ubuntu-latest` — no macOS needs).

1. Checkout
2. Download both artifacts
3. Read SHA256 files into job outputs: `sha256_arm64`, `sha256_x86_64`
4. `gh release create ${{ github.ref_name }}` with:
   - `--generate-notes`
   - Both `.tar.gz` files attached
   - `checksums.txt` attached (concatenation of both sha256 files, formatted `<sha> <filename>`)
5. Set job outputs for downstream formula update job.

#### Job C: `update-formula` (depends on `publish`)

Runs on `ubuntu-latest`.

1. Checkout `main` (fresh clone with `ref: main`, not the tag) using `token: ${{ secrets.GITHUB_TOKEN }}`
2. Render formula from template:
   - `sed` substitute `{{VERSION}}`, `{{ARM64_SHA256}}`, `{{X86_64_SHA256}}` in `Homebrew/murmur.rb.tmpl`
   - Write result to `Homebrew/murmur.rb`
3. Configure git author as `github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>`
4. `git commit -am "chore: update Homebrew formula to <version> [skip ci]"`
5. `git push origin main` — retry once on conflict (fetch + rebase + push)

Auth: uses default `GITHUB_TOKEN` with `contents: write` permission. No PAT needed.

**Formula-rewrite strategy:** `Homebrew/murmur.rb.tmpl` is the source of truth, containing the same content as `murmur.rb` but with `{{VERSION}}`, `{{ARM64_SHA256}}`, and `{{X86_64_SHA256}}` placeholder tokens. On each release, Job C substitutes these tokens with the real values (via `sed`) and writes the result to `Homebrew/murmur.rb`. This makes the update step deterministic and idempotent — no regex matching against the previous state of the formula, works identically on the first release and the hundredth.

### Workflow 3: `formula-test.yml`

**Trigger:** `pull_request` affecting `Homebrew/murmur.rb` or `Homebrew/murmur.rb.tmpl`.

Runs on `macos-14`:

1. Checkout
2. `brew install --formula ./Homebrew/murmur.rb` (will fetch the release tarball)
3. `murmur --version` — smoke test
4. `brew audit --strict --formula ./Homebrew/murmur.rb`

Note: this only runs meaningfully *after* a release has happened (since the formula references tarballs that must exist). For PRs that bump the formula ahead of a release, `brew install` will fail — which is correct behavior.

### Secrets

Pushed via `gh secret set` to `matovu-farid/murmur`:

| Secret | Source |
|---|---|
| `APPLE_CERTIFICATE_P12_BASE64` | `.p12` export of Developer ID cert, base64-encoded |
| `APPLE_CERTIFICATE_PASSWORD` | User-chosen password at `.p12` export time |
| `KEYCHAIN_PASSWORD` | Auto-generated 32-char random string |
| `APPLE_TEAM_ID` | `9VL7VRY6QZ` |
| `APP_STORE_CONNECT_KEY_ID` | `ZD9M92A6PY` |
| `APP_STORE_CONNECT_ISSUER_ID` | From App Store Connect API page (user provides) |
| `APP_STORE_CONNECT_KEY_P8` | `AuthKey_ZD9M92A6PY.p8` base64-encoded |

Cert export procedure (interactive, user runs in their terminal):

```
security export -k login.keychain-db \
  -t identities \
  -f pkcs12 \
  -P "<user-chosen-export-password>" \
  -o $TMPDIR/murmur-signing.p12
```

User enters macOS login password when prompted to unlock the private key. Then the `.p12` is base64-encoded, pushed as a secret, and the plaintext `.p12` deleted.

### Release flow (end-to-end)

1. User bumps `version` in `Cargo.toml`, commits on `main`
2. User tags: `git tag v0.2.0 && git push --tags`
3. `release.yml` fires:
   - Matrix builds arm64 + x86_64 in parallel
   - Each signed with Developer ID + notarized (~2-5 min per arch)
   - GitHub Release created with both tarballs + checksums
   - Formula auto-updated and pushed to `main` with `[skip ci]`
4. End users: `brew tap matovu-farid/murmur https://github.com/matovu-farid/murmur && brew install matovu-farid/murmur/murmur`

No Gatekeeper warnings (notarized). First run of any release binary triggers an online notarization check; subsequent runs are cached.

### Failure handling

- **Notarization failure:** `notarytool submit --wait` exits non-zero → job fails → no release published → no formula commit. User inspects notarization log (`xcrun notarytool log <submission-id>`), fixes, re-tags.
- **Cert expired/missing:** `security import` fails → job fails fast before any build work.
- **Formula push race:** if someone pushed to `main` between checkout and push, retry once with fetch + rebase. If still conflicts, fail loudly — human resolves.
- **Tag already exists:** `git tag` refuses locally. If tag push races, GitHub rejects — user deletes and retries.

### Testing before first real release

Dry-run with a pre-release tag to verify the full pipeline end-to-end:

```
git tag v0.0.1-test
git push --tags
# ... wait for workflow to complete ...
gh release delete v0.0.1-test --cleanup-tag --yes
```

Revert any formula auto-update commit on `main` manually if needed.

## Implementation order

1. Gather remaining credentials from user: Issuer ID + `.p12` cert export
2. Create public repo `matovu-farid/murmur` via `gh repo create --source . --push`
3. Push all GitHub secrets via `gh secret set`
4. Add `rust-toolchain.toml` pinning `stable` + `rustfmt` + `clippy`
5. Create `Homebrew/murmur.rb.tmpl` (copy of current `murmur.rb` with `OWNER` → `matovu-farid` and `REPLACE_WITH_*_SHA256` → `{{ARM64_SHA256}}` / `{{X86_64_SHA256}}` / `{{VERSION}}` tokens)
6. Update `Homebrew/murmur.rb` to replace `OWNER` → `matovu-farid` (keeps current placeholder SHAs until first real release)
7. Add `.github/workflows/ci.yml`
8. Verify CI passes on a no-op PR
9. Add `.github/workflows/release.yml` and `.github/workflows/formula-test.yml`
10. Enable branch protection on `main` requiring `ci` workflow
11. Dry-run release with `v0.0.1-test`, verify end-to-end (signed, notarized, formula auto-updated)
12. Clean up test release (`gh release delete v0.0.1-test --cleanup-tag --yes`) and revert test formula-update commit

## Success criteria

- Pushing to `main` or opening a PR triggers `ci.yml` and it passes green on a clean codebase
- Tagging `v0.0.1-test` produces a GitHub Release with two notarized tarballs and valid SHA256 checksums
- `brew install --formula ./Homebrew/murmur.rb` from a fresh clone installs and runs `murmur --version` successfully on both arm64 and x86_64 machines
- `codesign --verify --strict` passes on downloaded release binaries
- `spctl --assess --type execute` passes on downloaded release binaries (proves notarization)
- Formula auto-updates on subsequent tags without human intervention
- No secrets appear in workflow logs (all masked via `::add-mask::` where necessary)
