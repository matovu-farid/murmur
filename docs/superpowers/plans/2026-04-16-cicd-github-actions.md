# CI/CD with GitHub Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire up GitHub Actions workflows for `murmur` providing CI on every push/PR and a fully-automated signed + notarized macOS release flow triggered by git tags, with in-repo Homebrew formula auto-updates.

**Architecture:** Three workflows — `ci.yml` (test/lint/build on every change), `release.yml` (matrix build → sign → notarize → GitHub Release → auto-update formula), `formula-test.yml` (brew audit on formula PRs). Uses App Store Connect API key for notarization and Developer ID cert for signing, both injected via GitHub secrets. Formula rendering is template-based (`.tmpl` → `sed` substitution) for deterministic updates.

**Tech Stack:** GitHub Actions, macOS runners (`macos-14` arm64 + `macos-13` x86_64), `codesign`, `xcrun notarytool`, `security` (keychain), Rust stable toolchain, Homebrew formula DSL.

**Reference spec:** `docs/superpowers/specs/2026-04-16-cicd-github-actions-design.md`

---

## Known inputs (from brainstorming)

- Repo: `matovu-farid/murmur` (public, to be created)
- App Store Connect API Key ID: `ZD9M92A6PY`
- App Store Connect Issuer ID: `ec064019-96d5-44fd-9840-dcc6a134c92e`
- Apple Team ID: `9VL7VRY6QZ`
- Developer ID cert CN: `Developer ID Application: Farid Matovu (9VL7VRY6QZ)` (in login keychain, valid through Feb 1, 2027)
- API Key file: `/Users/faridmatovu/projects/wipr/AuthKey_ZD9M92A6PY.p8` (gitignored)

---

## File Structure

### New files

- `rust-toolchain.toml` — pins Rust channel + components for reproducible builds
- `.github/workflows/ci.yml` — CI workflow
- `.github/workflows/release.yml` — release workflow
- `.github/workflows/formula-test.yml` — Homebrew formula test workflow
- `Homebrew/murmur.rb.tmpl` — formula template with `{{VERSION}}`, `{{ARM64_SHA256}}`, `{{X86_64_SHA256}}` tokens; source of truth

### Modified files

- `Homebrew/murmur.rb` — `OWNER` → `matovu-farid` (one-time); auto-regenerated from template on every release
- `.gitignore` — already updated in brainstorming phase; includes `*.p8`, `*.p12`, `.env*`

### No Rust source changes

This plan adds only infrastructure. No changes to `src/`.

---

## Task 1: Export signing certificate to .p12

**Purpose:** Convert the Developer ID cert installed in the user's macOS login keychain into a base64-encoded `.p12` that can be injected into CI keychains.

**Files:**
- Create: `$TMPDIR/murmur-signing.p12` (temp, deleted after)
- Reference: Developer ID cert already in user's login keychain

- [ ] **Step 1: Generate a strong export password**

Run:
```bash
openssl rand -base64 24
```

Save the output to a note — this becomes the `APPLE_CERTIFICATE_PASSWORD` secret. Example format: `K4x9pQ+mR2vNt7wLhE6uJ8aY`.

Expected: A 32-char base64 string. This password protects the `.p12` in transit (stored once in GitHub secrets, never reused).

- [ ] **Step 2: Export the cert and its private key to .p12**

Run (user runs this interactively — will be prompted for macOS login password multiple times to unlock the private key):

```bash
security export \
  -k ~/Library/Keychains/login.keychain-db \
  -t identities \
  -f pkcs12 \
  -P "<EXPORT_PASSWORD_FROM_STEP_1>" \
  -o "$TMPDIR/murmur-signing.p12"
```

Expected: file `$TMPDIR/murmur-signing.p12` is created, size ~3-4KB.

Note: `-t identities` exports ALL code-signing identities. If the user has multiple Developer ID certs, we'll need a more surgical approach using `-c` with the common name. For now, assume the single "Developer ID Application: Farid Matovu" identity shown in the earlier `find-identity -v` output.

- [ ] **Step 3: Verify the .p12 is valid and contains the expected cert**

Run:
```bash
openssl pkcs12 -in "$TMPDIR/murmur-signing.p12" -passin pass:"<EXPORT_PASSWORD_FROM_STEP_1>" -nokeys -info 2>/dev/null | openssl x509 -noout -subject
```

Expected output:
```
subject=UID=9VL7VRY6QZ, CN=Developer ID Application: Farid Matovu (9VL7VRY6QZ), OU=9VL7VRY6QZ, O=Farid Matovu, C=US
```

If subject shows a different CN, export extracted the wrong identity — redo with `-c "Developer ID Application: Farid Matovu"`.

- [ ] **Step 4: Base64-encode the .p12 for GitHub secret storage**

Run:
```bash
base64 -i "$TMPDIR/murmur-signing.p12" | tr -d '\n' > "$TMPDIR/murmur-signing.p12.b64"
wc -c < "$TMPDIR/murmur-signing.p12.b64"
```

Expected: file exists, size roughly 4500-5500 chars (base64 is ~1.37x binary).

- [ ] **Step 5: Do NOT commit anything yet**

This task produces only credential artifacts in `$TMPDIR`. No git operation. The `.p12.b64` and export password will be pushed as GitHub secrets in Task 6.

Note: `$TMPDIR` on macOS is a per-user directory (typically `/var/folders/…/T/`) that's cleaned up on reboot. Do not move these files into the project directory.

---

## Task 2: Create the public GitHub repo and push current history

**Files:**
- External: GitHub repo `matovu-farid/murmur` (created via `gh repo create`)

- [ ] **Step 1: Confirm git state is clean**

Run:
```bash
cd /Users/faridmatovu/projects/wipr
git status
```

Expected: `On branch main`, `nothing to commit, working tree clean`.

If dirty, investigate before proceeding — we want a clean initial push. The `.gitignore` + spec commits from brainstorming should already be committed.

- [ ] **Step 2: Create the repo and push `main`**

Run:
```bash
gh repo create matovu-farid/murmur \
  --public \
  --source=/Users/faridmatovu/projects/wipr \
  --description "macOS voice-to-text dictation CLI daemon" \
  --push
```

Expected output:
```
✓ Created repository matovu-farid/murmur on github.com
✓ Added remote https://github.com/matovu-farid/murmur.git
✓ Pushed commits to https://github.com/matovu-farid/murmur.git
```

- [ ] **Step 3: Verify the push**

Run:
```bash
gh repo view matovu-farid/murmur --json name,visibility,defaultBranchRef
git remote -v
```

Expected: repo exists, public, default branch `main`, origin points to `https://github.com/matovu-farid/murmur.git`.

- [ ] **Step 4: No commit needed**

This task creates an external resource. Local state is unchanged.

---

## Task 3: Replace OWNER placeholder in Homebrew/murmur.rb

**Files:**
- Modify: `Homebrew/murmur.rb` (3 occurrences of `OWNER`)

- [ ] **Step 1: Replace OWNER with matovu-farid**

Edit `Homebrew/murmur.rb`. The file currently has:

```ruby
  homepage "https://github.com/OWNER/murmur"
  ...
      url "https://github.com/OWNER/murmur/releases/download/v#{version}/murmur-#{version}-aarch64-apple-darwin.tar.gz"
  ...
      url "https://github.com/OWNER/murmur/releases/download/v#{version}/murmur-#{version}-x86_64-apple-darwin.tar.gz"
```

Replace all three `OWNER` with `matovu-farid`. Final file (full contents):

```ruby
class Murmur < Formula
  desc "macOS voice-to-text dictation daemon"
  homepage "https://github.com/matovu-farid/murmur"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/matovu-farid/murmur/releases/download/v#{version}/murmur-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end

    on_intel do
      url "https://github.com/matovu-farid/murmur/releases/download/v#{version}/murmur-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_SHA256"
    end
  end

  def install
    bin.install "murmur"
  end

  def caveats
    <<~EOS
      Murmur needs Input Monitoring permission to detect the fn key.
      Open System Settings → Privacy & Security → Input Monitoring,
      then enable Murmur after first run.

      To start the daemon:
        murmur start

      To install for auto-start on login:
        murmur install

      To configure interactively:
        murmur config
    EOS
  end

  test do
    assert_match "murmur", shell_output("#{bin}/murmur --version")
  end
end
```

The `REPLACE_WITH_*_SHA256` placeholders remain — the release workflow will overwrite the entire file from the template on first real release. Before that first release, the formula is non-functional for `brew install`, which is fine (no users yet).

- [ ] **Step 2: Verify the change**

Run:
```bash
grep -c "OWNER" Homebrew/murmur.rb
grep -c "matovu-farid" Homebrew/murmur.rb
```

Expected: `OWNER` count is `0`, `matovu-farid` count is `3`.

- [ ] **Step 3: Commit**

```bash
git add Homebrew/murmur.rb
git commit -m "chore: set GitHub owner in Homebrew formula"
```

Expected: one commit on `main`, one file changed.

---

## Task 4: Create Homebrew/murmur.rb.tmpl (source of truth for releases)

**Files:**
- Create: `Homebrew/murmur.rb.tmpl`

- [ ] **Step 1: Create the template file**

Create `Homebrew/murmur.rb.tmpl` with this exact content:

```ruby
class Murmur < Formula
  desc "macOS voice-to-text dictation daemon"
  homepage "https://github.com/matovu-farid/murmur"
  version "{{VERSION}}"

  on_macos do
    on_arm do
      url "https://github.com/matovu-farid/murmur/releases/download/v#{version}/murmur-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "{{ARM64_SHA256}}"
    end

    on_intel do
      url "https://github.com/matovu-farid/murmur/releases/download/v#{version}/murmur-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "{{X86_64_SHA256}}"
    end
  end

  def install
    bin.install "murmur"
  end

  def caveats
    <<~EOS
      Murmur needs Input Monitoring permission to detect the fn key.
      Open System Settings → Privacy & Security → Input Monitoring,
      then enable Murmur after first run.

      To start the daemon:
        murmur start

      To install for auto-start on login:
        murmur install

      To configure interactively:
        murmur config
    EOS
  end

  test do
    assert_match "murmur", shell_output("#{bin}/murmur --version")
  end
end
```

The tokens `{{VERSION}}`, `{{ARM64_SHA256}}`, `{{X86_64_SHA256}}` are what the release workflow will replace via `sed`.

- [ ] **Step 2: Verify no accidental edits**

Run:
```bash
grep -c "{{VERSION}}" Homebrew/murmur.rb.tmpl
grep -c "{{ARM64_SHA256}}" Homebrew/murmur.rb.tmpl
grep -c "{{X86_64_SHA256}}" Homebrew/murmur.rb.tmpl
```

Expected: each count is `1`.

- [ ] **Step 3: Commit**

```bash
git add Homebrew/murmur.rb.tmpl
git commit -m "chore: add Homebrew formula template for release automation"
```

---

## Task 5: Add rust-toolchain.toml

**Purpose:** Pin Rust channel and required components so every CI run and every developer gets an identical toolchain.

**Files:**
- Create: `rust-toolchain.toml`

- [ ] **Step 1: Create the file**

Create `rust-toolchain.toml` at the repo root with this exact content:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

Notes:
- `channel = "stable"` — floats to latest stable. Can be pinned to `"1.XX.Y"` later if reproducibility against a specific version is needed.
- `components` — needed for `cargo fmt --check` and `cargo clippy` in CI.
- `profile = "minimal"` — skips rust-docs etc., shaving install time.

- [ ] **Step 2: Verify local toolchain still works**

Run:
```bash
rustup show active-toolchain
cargo --version
cargo fmt -- --version
cargo clippy --version
```

Expected: `stable-aarch64-apple-darwin (overridden by '/Users/faridmatovu/projects/wipr/rust-toolchain.toml')` and all three commands succeed.

If `cargo fmt` or `cargo clippy` errors saying the component isn't installed, run `rustup component add rustfmt clippy` first.

- [ ] **Step 3: Commit**

```bash
git add rust-toolchain.toml
git commit -m "chore: pin Rust toolchain to stable with rustfmt and clippy"
```

---

## Task 6: Push all GitHub secrets

**Purpose:** Make all signing + notarization credentials available to the release workflow.

**Files:**
- External: GitHub secrets on `matovu-farid/murmur`

- [ ] **Step 1: Generate `KEYCHAIN_PASSWORD`**

Run:
```bash
KEYCHAIN_PWD=$(openssl rand -base64 32 | tr -d '\n')
echo "$KEYCHAIN_PWD" | wc -c
```

Expected: 44 chars. Capture `$KEYCHAIN_PWD` for step 2. This password protects the ephemeral build-time keychain in CI; it never persists anywhere except GitHub secrets.

- [ ] **Step 2: Push all seven secrets**

Run each of these. The values come from Task 1 (two), known constants (three), and files (two):

```bash
# Known constants
gh secret set APPLE_TEAM_ID --body "9VL7VRY6QZ" --repo matovu-farid/murmur
gh secret set APP_STORE_CONNECT_KEY_ID --body "ZD9M92A6PY" --repo matovu-farid/murmur
gh secret set APP_STORE_CONNECT_ISSUER_ID --body "ec064019-96d5-44fd-9840-dcc6a134c92e" --repo matovu-farid/murmur

# Generated
gh secret set KEYCHAIN_PASSWORD --body "$KEYCHAIN_PWD" --repo matovu-farid/murmur

# User-chosen export password from Task 1 Step 1
gh secret set APPLE_CERTIFICATE_PASSWORD --body "<EXPORT_PASSWORD_FROM_TASK_1>" --repo matovu-farid/murmur

# Cert bytes from Task 1 Step 4 (read from file via stdin)
gh secret set APPLE_CERTIFICATE_P12_BASE64 \
  --repo matovu-farid/murmur \
  < "$TMPDIR/murmur-signing.p12.b64"

# API key bytes — base64 the .p8 and push via stdin
base64 -i /Users/faridmatovu/projects/wipr/AuthKey_ZD9M92A6PY.p8 | tr -d '\n' | \
  gh secret set APP_STORE_CONNECT_KEY_P8 --repo matovu-farid/murmur
```

Expected: each command prints `✓ Set Actions secret <NAME> for matovu-farid/murmur`.

- [ ] **Step 3: Verify all seven secrets exist**

Run:
```bash
gh secret list --repo matovu-farid/murmur
```

Expected output (order may vary):
```
APPLE_CERTIFICATE_P12_BASE64  Updated ...
APPLE_CERTIFICATE_PASSWORD    Updated ...
APPLE_TEAM_ID                 Updated ...
APP_STORE_CONNECT_ISSUER_ID   Updated ...
APP_STORE_CONNECT_KEY_ID      Updated ...
APP_STORE_CONNECT_KEY_P8      Updated ...
KEYCHAIN_PASSWORD             Updated ...
```

Seven total. If any are missing, re-run the corresponding command.

- [ ] **Step 4: Delete the local .p12 and .p12.b64 files**

The cert is now in GitHub secrets; the local files are no longer needed and shouldn't linger.

```bash
rm -f "$TMPDIR/murmur-signing.p12" "$TMPDIR/murmur-signing.p12.b64"
ls "$TMPDIR/murmur-signing."* 2>/dev/null || echo "Cleaned up."
```

Expected: `Cleaned up.`

- [ ] **Step 5: No git commit needed**

Secrets are stored externally in GitHub. Local state unchanged.

---

## Task 7: Create CI workflow (ci.yml)

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create the workflow file**

Create `.github/workflows/ci.yml` with this exact content:

```yaml
name: ci

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  test:
    name: test (${{ matrix.target }})
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: macos-14
            target: aarch64-apple-darwin

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
          components: rustfmt, clippy

      - name: Cache cargo registry and build artifacts
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      - name: cargo fmt --check
        run: cargo fmt --all -- --check

      - name: cargo clippy
        run: cargo clippy --all-targets --target ${{ matrix.target }} -- -D warnings

      - name: cargo test
        run: cargo test --all --target ${{ matrix.target }}

      - name: cargo build --release
        run: cargo build --release --target ${{ matrix.target }}
```

Notes:
- Single matrix entry for now (arm64). If cross-arch CI becomes valuable, add `{ runner: macos-13, target: x86_64-apple-darwin }`.
- `fail-fast: false` keeps future matrix entries independent.
- `permissions: contents: read` is the minimum (checkout) — follows principle of least privilege.
- `concurrency` with `cancel-in-progress: true` kills prior runs on force-push/rapid pushes.

- [ ] **Step 2: Validate the YAML syntax locally**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "YAML OK"
```

Expected: `YAML OK`.

If you have `actionlint` installed (`brew install actionlint`), also run:
```bash
actionlint .github/workflows/ci.yml
```

Expected: no output (success).

- [ ] **Step 3: Commit and push to trigger the first CI run**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add cargo fmt, clippy, test, build workflow"
git push origin main
```

- [ ] **Step 4: Watch the first run**

Run:
```bash
gh run watch --repo matovu-farid/murmur
```

Expected: run starts on `macos-14`, completes green. Total wall time: ~8-12 min first run (cold cache), ~2-3 min subsequent.

If CI fails:
- **`cargo fmt --check` fails** → run `cargo fmt` locally and commit.
- **`cargo clippy` fails** → fix warnings, commit.
- **`cargo test` fails** → investigate locally with `cargo test` and fix before continuing.
- **Build-only failure** → almost always a feature flag or target-specific issue.

Do not proceed to Task 8 until CI is green on `main`.

- [ ] **Step 5: Confirm green**

Run:
```bash
gh run list --workflow ci.yml --repo matovu-farid/murmur --limit 1
```

Expected: status `completed`, conclusion `success`.

---

## Task 8: Create release workflow (release.yml)

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Create the workflow file**

Create `.github/workflows/release.yml` with this exact content:

```yaml
name: release

on:
  push:
    tags:
      - 'v*.*.*'
      - 'v*.*.*-*'   # pre-release tags like v0.0.1-test

permissions:
  contents: write   # required for gh release create and pushing formula update

jobs:
  build:
    name: build (${{ matrix.target }})
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - runner: macos-14
            target: aarch64-apple-darwin
          - runner: macos-13
            target: x86_64-apple-darwin

    outputs:
      version: ${{ steps.version.outputs.version }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Extract version from tag
        id: version
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
          echo "Version: $VERSION"

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Cache cargo registry and build artifacts
        uses: Swatinem/rust-cache@v2
        with:
          key: release-${{ matrix.target }}

      - name: Create temporary keychain and import signing cert
        env:
          APPLE_CERTIFICATE_P12_BASE64: ${{ secrets.APPLE_CERTIFICATE_P12_BASE64 }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          KEYCHAIN_PASSWORD: ${{ secrets.KEYCHAIN_PASSWORD }}
        run: |
          set -euo pipefail
          CERT_PATH="$RUNNER_TEMP/cert.p12"
          KEYCHAIN_PATH="$RUNNER_TEMP/build.keychain-db"
          echo "$APPLE_CERTIFICATE_P12_BASE64" | base64 -d > "$CERT_PATH"
          security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security set-keychain-settings -lut 3600 "$KEYCHAIN_PATH"
          security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security import "$CERT_PATH" -k "$KEYCHAIN_PATH" \
            -P "$APPLE_CERTIFICATE_PASSWORD" \
            -T /usr/bin/codesign \
            -T /usr/bin/security
          security list-keychains -d user -s "$KEYCHAIN_PATH" $(security list-keychains -d user | tr -d '"')
          security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security find-identity -v -p codesigning "$KEYCHAIN_PATH"

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Sign binary
        env:
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          set -euo pipefail
          BIN="target/${{ matrix.target }}/release/murmur"
          codesign \
            --sign "Developer ID Application: Farid Matovu (${APPLE_TEAM_ID})" \
            --options runtime \
            --timestamp \
            --force \
            "$BIN"
          codesign --verify --strict --verbose=2 "$BIN"

      - name: Create .zip for notarization
        run: |
          set -euo pipefail
          BIN="target/${{ matrix.target }}/release/murmur"
          ditto -c -k --keepParent "$BIN" "$RUNNER_TEMP/murmur-notarize.zip"

      - name: Submit for notarization
        env:
          APP_STORE_CONNECT_KEY_P8: ${{ secrets.APP_STORE_CONNECT_KEY_P8 }}
          APP_STORE_CONNECT_KEY_ID: ${{ secrets.APP_STORE_CONNECT_KEY_ID }}
          APP_STORE_CONNECT_ISSUER_ID: ${{ secrets.APP_STORE_CONNECT_ISSUER_ID }}
        run: |
          set -euo pipefail
          KEY_PATH="$RUNNER_TEMP/key.p8"
          echo "$APP_STORE_CONNECT_KEY_P8" | base64 -d > "$KEY_PATH"
          chmod 600 "$KEY_PATH"
          xcrun notarytool submit "$RUNNER_TEMP/murmur-notarize.zip" \
            --key "$KEY_PATH" \
            --key-id "$APP_STORE_CONNECT_KEY_ID" \
            --issuer "$APP_STORE_CONNECT_ISSUER_ID" \
            --wait \
            --timeout 20m

      - name: Package for distribution
        run: |
          set -euo pipefail
          VERSION="${{ steps.version.outputs.version }}"
          TARGET="${{ matrix.target }}"
          STAGE="$RUNNER_TEMP/stage"
          mkdir -p "$STAGE"
          cp "target/$TARGET/release/murmur" "$STAGE/murmur"
          tar -C "$STAGE" -czf "murmur-${VERSION}-${TARGET}.tar.gz" murmur
          shasum -a 256 "murmur-${VERSION}-${TARGET}.tar.gz" | awk '{print $1}' > "${TARGET}.sha256"
          echo "Tarball:"
          ls -la "murmur-${VERSION}-${TARGET}.tar.gz"
          echo "SHA256: $(cat ${TARGET}.sha256)"

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: |
            murmur-${{ steps.version.outputs.version }}-${{ matrix.target }}.tar.gz
            ${{ matrix.target }}.sha256
          if-no-files-found: error
          retention-days: 7

      - name: Cleanup keychain and credentials
        if: always()
        run: |
          security delete-keychain "$RUNNER_TEMP/build.keychain-db" 2>/dev/null || true
          rm -f "$RUNNER_TEMP/cert.p12" "$RUNNER_TEMP/key.p8" "$RUNNER_TEMP/murmur-notarize.zip"

  publish:
    name: publish release
    needs: build
    runs-on: ubuntu-latest
    outputs:
      arm64_sha: ${{ steps.shas.outputs.arm64_sha }}
      x86_64_sha: ${{ steps.shas.outputs.x86_64_sha }}
      version: ${{ needs.build.outputs.version }}
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Read SHA256 values
        id: shas
        run: |
          set -euo pipefail
          ARM64_SHA=$(cat artifacts/aarch64-apple-darwin/aarch64-apple-darwin.sha256)
          X86_64_SHA=$(cat artifacts/x86_64-apple-darwin/x86_64-apple-darwin.sha256)
          echo "arm64_sha=$ARM64_SHA" >> "$GITHUB_OUTPUT"
          echo "x86_64_sha=$X86_64_SHA" >> "$GITHUB_OUTPUT"
          echo "arm64:  $ARM64_SHA"
          echo "x86_64: $X86_64_SHA"

      - name: Build checksums.txt
        run: |
          set -euo pipefail
          VERSION="${{ needs.build.outputs.version }}"
          ARM64_SHA="${{ steps.shas.outputs.arm64_sha }}"
          X86_64_SHA="${{ steps.shas.outputs.x86_64_sha }}"
          {
            echo "${ARM64_SHA}  murmur-${VERSION}-aarch64-apple-darwin.tar.gz"
            echo "${X86_64_SHA}  murmur-${VERSION}-x86_64-apple-darwin.tar.gz"
          } > checksums.txt
          cat checksums.txt

      - name: Create GitHub Release
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          set -euo pipefail
          VERSION="${{ needs.build.outputs.version }}"
          gh release create "v${VERSION}" \
            --repo "$GITHUB_REPOSITORY" \
            --title "v${VERSION}" \
            --generate-notes \
            "artifacts/aarch64-apple-darwin/murmur-${VERSION}-aarch64-apple-darwin.tar.gz" \
            "artifacts/x86_64-apple-darwin/murmur-${VERSION}-x86_64-apple-darwin.tar.gz" \
            "checksums.txt"

  update-formula:
    name: update Homebrew formula
    needs: publish
    runs-on: ubuntu-latest
    # Skip formula update for pre-release tags (e.g. v0.0.1-test)
    if: ${{ !contains(needs.publish.outputs.version, '-') }}
    steps:
      - name: Checkout main
        uses: actions/checkout@v4
        with:
          ref: main
          token: ${{ secrets.GITHUB_TOKEN }}
          fetch-depth: 0

      - name: Render formula from template
        run: |
          set -euo pipefail
          VERSION="${{ needs.publish.outputs.version }}"
          ARM64_SHA="${{ needs.publish.outputs.arm64_sha }}"
          X86_64_SHA="${{ needs.publish.outputs.x86_64_sha }}"
          sed \
            -e "s|{{VERSION}}|${VERSION}|g" \
            -e "s|{{ARM64_SHA256}}|${ARM64_SHA}|g" \
            -e "s|{{X86_64_SHA256}}|${X86_64_SHA}|g" \
            Homebrew/murmur.rb.tmpl > Homebrew/murmur.rb
          echo "--- Rendered formula ---"
          cat Homebrew/murmur.rb

      - name: Commit and push
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          set -euo pipefail
          VERSION="${{ needs.publish.outputs.version }}"
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

          if git diff --quiet Homebrew/murmur.rb; then
            echo "No formula changes — nothing to commit."
            exit 0
          fi

          git add Homebrew/murmur.rb
          git commit -m "chore: update Homebrew formula to v${VERSION} [skip ci]"

          # Retry-once on push conflict
          if ! git push origin main; then
            echo "Push conflict, retrying with rebase..."
            git pull --rebase origin main
            git push origin main
          fi
```

Explanation of key choices:
- Matrix uses two runner types (`macos-14` for arm64 native, `macos-13` for x86_64 native) — avoids cross-compiling `whisper-rs`'s C deps.
- `permissions: contents: write` at top level — needed for `gh release create` and pushing formula update.
- Notarization uses `notarytool --wait --timeout 20m` — Apple's typical notarization latency is 1-5 min; 20 min is generous headroom.
- Formula-update job is skipped for pre-release tags (containing `-`) so our dry-run in Task 11 doesn't corrupt `main`'s formula.
- Cleanup step runs with `if: always()` so secrets are wiped even if an earlier step fails.
- No `strict_required_status_checks` gating here; branch protection (Task 10) handles that.

- [ ] **Step 2: Validate YAML**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML OK"
actionlint .github/workflows/release.yml 2>/dev/null || echo "actionlint not installed, skipping"
```

Expected: `YAML OK`. If actionlint is installed, no errors.

- [ ] **Step 3: Commit and push**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add signed + notarized macOS release workflow"
git push origin main
```

- [ ] **Step 4: Verify the workflow is registered**

Run:
```bash
gh workflow list --repo matovu-farid/murmur
```

Expected:
```
NAME     STATE   ID
ci       active  ...
release  active  ...
```

The release workflow won't run yet (no tag pushed) — that's expected. Tag-triggered workflows don't fire on code changes.

---

## Task 9: Create formula-test workflow (formula-test.yml)

**Files:**
- Create: `.github/workflows/formula-test.yml`

- [ ] **Step 1: Create the workflow file**

Create `.github/workflows/formula-test.yml` with this exact content:

```yaml
name: formula-test

on:
  pull_request:
    paths:
      - 'Homebrew/murmur.rb'
      - 'Homebrew/murmur.rb.tmpl'
      - '.github/workflows/formula-test.yml'

permissions:
  contents: read

jobs:
  test-formula:
    name: brew install + audit
    runs-on: macos-14
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Brew audit
        run: brew audit --strict --formula ./Homebrew/murmur.rb

      - name: Brew install (will fetch release tarball)
        run: brew install --formula ./Homebrew/murmur.rb

      - name: Verify binary runs
        run: murmur --version
```

Notes:
- Only runs on PRs that touch the formula file or this workflow.
- `brew install` here will fail on PRs that bump the formula to a version whose release artifacts don't exist yet. That's correct — it catches PRs that race the release process.
- `brew audit --strict` catches Ruby/formula syntax issues.

- [ ] **Step 2: Validate YAML**

Run:
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/formula-test.yml'))" && echo "YAML OK"
```

Expected: `YAML OK`.

- [ ] **Step 3: Commit and push**

```bash
git add .github/workflows/formula-test.yml
git commit -m "ci: add Homebrew formula install and audit workflow"
git push origin main
```

- [ ] **Step 4: Verify the workflow is registered**

Run:
```bash
gh workflow list --repo matovu-farid/murmur
```

Expected: three workflows — `ci`, `release`, `formula-test`.

---

## Task 10: Configure branch protection ruleset for main

**Purpose:** Require `ci` workflow to pass before a PR can merge to `main`. Intentionally NOT requiring a PR for every push to `main`, because the release workflow's formula-update job pushes directly to `main` as `github-actions[bot]` using `GITHUB_TOKEN`. The `required_status_checks` rule in GitHub rulesets only gates *PR merges*, not direct pushes — exactly the behavior we want.

**Trade-off:** A human admin could also push directly to `main` and bypass CI. For a solo project this is acceptable; in practice the user will use PRs for any code change (habit). If the project gains collaborators later, switch to the stricter config described in the "Hardening" note at the end of this task.

**Files:**
- External: branch ruleset on `matovu-farid/murmur`

- [ ] **Step 1: Create the ruleset via gh api**

Run:
```bash
gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  /repos/matovu-farid/murmur/rulesets \
  --input - <<'EOF'
{
  "name": "main-protection",
  "target": "branch",
  "enforcement": "active",
  "conditions": {
    "ref_name": {
      "include": ["refs/heads/main"],
      "exclude": []
    }
  },
  "rules": [
    {
      "type": "required_status_checks",
      "parameters": {
        "required_status_checks": [
          { "context": "test (aarch64-apple-darwin)" }
        ],
        "strict_required_status_checks_policy": false
      }
    },
    { "type": "non_fast_forward" },
    { "type": "deletion" }
  ],
  "bypass_actors": []
}
EOF
```

Explanation of rules:
- `required_status_checks` — PRs targeting `main` must have the `test (aarch64-apple-darwin)` check green before they can be merged. Does not affect direct pushes.
- `non_fast_forward` — prevents `git push --force` to `main`.
- `deletion` — prevents deleting the `main` branch.
- No `pull_request` rule — direct pushes to `main` are allowed (needed for formula-update bot commits).
- Empty `bypass_actors` — no one bypasses these rules (there's nothing to bypass for bot pushes since we don't require PRs).

- [ ] **Step 2: Verify the ruleset is active**

Run:
```bash
gh api /repos/matovu-farid/murmur/rulesets --jq '.[] | {id, name, enforcement, target}'
```

Expected:
```json
{"id": ..., "name": "main-protection", "enforcement": "active", "target": "branch"}
```

- [ ] **Step 3: Confirm the required check name matches the actual CI job name**

Run:
```bash
gh api /repos/matovu-farid/murmur/actions/workflows/ci.yml/runs --jq '.workflow_runs[0].id' | \
  xargs -I {} gh api /repos/matovu-farid/murmur/actions/runs/{}/jobs --jq '.jobs[].name'
```

Expected output includes a job named exactly `test (aarch64-apple-darwin)`. If the rendered name differs, update the ruleset by:
1. `RULESET_ID=$(gh api /repos/matovu-farid/murmur/rulesets --jq '.[0].id')`
2. Re-run the POST in Step 1 as a `PUT` to `/repos/matovu-farid/murmur/rulesets/$RULESET_ID` with the corrected `context` string.

- [ ] **Step 4: No git commit needed**

Rulesets are stored externally. Local state unchanged.

**Hardening note (for future when collaborators join):**
If you later want to enforce that even admin humans must go through PRs, add a `pull_request` rule AND add `github-actions[bot]` to `bypass_actors` so formula-update still works:
```json
{
  "type": "pull_request",
  "parameters": { "required_approving_review_count": 0, ... }
}
```
And lookup the Actions app actor_id via:
```bash
gh api /repos/matovu-farid/murmur/actions/permissions
```
Then add to `bypass_actors`:
```json
{ "actor_id": <id>, "actor_type": "Integration", "bypass_mode": "always" }
```

---

## Task 11: Dry-run release with v0.0.1-test tag

**Purpose:** End-to-end validation of the release workflow without affecting production formula state.

**Files:**
- External: pre-release tag `v0.0.1-test` on `matovu-farid/murmur`

- [ ] **Step 1: Confirm `main` is up to date and clean**

Run:
```bash
git fetch origin
git log --oneline origin/main -5
git status
```

Expected: working tree clean, local `main` matches `origin/main`.

- [ ] **Step 2: Create and push the test tag**

Run:
```bash
git tag v0.0.1-test
git push origin v0.0.1-test
```

Expected: tag pushed. The `-test` suffix ensures the `update-formula` job is skipped (per the `if:` condition on that job).

- [ ] **Step 3: Watch the release workflow**

Run:
```bash
gh run watch --repo matovu-farid/murmur
```

Expected: `release` workflow starts. Two parallel `build` jobs (arm64, x86_64) each take ~10-15 min (cold cache; first release ever). Then `publish` runs (~1 min). `update-formula` should show as skipped.

If a step fails, read the log carefully:
- **`security import` fails with "item already exists"** → keychain setup bug, adjust keychain listing order
- **`codesign` fails with "no identity found"** → cert wasn't imported correctly; check `security find-identity` output in log
- **`notarytool submit` returns "Invalid"** → download submission log with `xcrun notarytool log <submission-id>`; common cause is missing `--options runtime` or missing hardened runtime entitlement
- **`gh release create` fails with "release already exists"** → manual cleanup: `gh release delete v0.0.1-test -y` and re-tag
- **`push origin main` in update-formula fails with "protected branch"** → ruleset bypass config issue; add Integration actor to bypass list (see Task 10 Step 1 note)

- [ ] **Step 4: Verify the GitHub Release was created correctly**

Run:
```bash
gh release view v0.0.1-test --repo matovu-farid/murmur
```

Expected: release exists with three assets:
- `murmur-0.0.1-test-aarch64-apple-darwin.tar.gz`
- `murmur-0.0.1-test-x86_64-apple-darwin.tar.gz`
- `checksums.txt`

- [ ] **Step 5: Download and verify a signed + notarized binary**

Run:
```bash
mkdir -p /tmp/murmur-verify && cd /tmp/murmur-verify
gh release download v0.0.1-test --repo matovu-farid/murmur --pattern '*-aarch64-apple-darwin.tar.gz'
tar xzf murmur-0.0.1-test-aarch64-apple-darwin.tar.gz
codesign --verify --strict --verbose=2 ./murmur
spctl --assess --type execute --verbose ./murmur || echo "(spctl may print 'rejected: source=no usable signature' for bare binaries; this is expected pre-Gatekeeper-v2. Notarization check happens when the binary runs.)"
./murmur --version
```

Expected:
- `codesign --verify` prints `valid on disk` and `satisfies its Designated Requirement`
- `./murmur --version` prints the version string

If `./murmur --version` errors with "unexpected argument '--version'", `Args` in `src/cli/mod.rs` needs `#[command(version)]`. That's a Rust code fix — separate PR; document and continue.

- [ ] **Step 6: Verify update-formula was skipped**

Run:
```bash
gh run list --workflow release.yml --repo matovu-farid/murmur --limit 1 --json jobs --jq '.[0].jobs[] | {name, conclusion}'
```

Expected: `update-formula` has `conclusion: "skipped"`. The `build` and `publish` jobs have `conclusion: "success"`.

- [ ] **Step 7: No git commit needed for the dry-run itself**

The dry-run produces only an external release + tag. Cleanup happens in Task 12.

---

## Task 12: Cleanup test release

**Files:**
- External: delete release + tag from `matovu-farid/murmur`

- [ ] **Step 1: Delete the GitHub Release and its tag**

Run:
```bash
gh release delete v0.0.1-test --repo matovu-farid/murmur --cleanup-tag --yes
```

Expected: output like `✓ Deleted release v0.0.1-test` and the tag is removed from the remote.

- [ ] **Step 2: Delete the local tag**

Run:
```bash
git tag -d v0.0.1-test
```

Expected: `Deleted tag 'v0.0.1-test'`.

- [ ] **Step 3: Verify everything is clean**

Run:
```bash
git tag -l
gh release list --repo matovu-farid/murmur
```

Expected: no tags locally, no releases remotely.

- [ ] **Step 4: Remove local verification artifacts**

Run:
```bash
rm -rf /tmp/murmur-verify
```

- [ ] **Step 5: No git commit needed**

---

## Post-implementation: how to cut a real release

The pipeline is now production-ready. When the user wants to ship:

1. Bump `version` in `Cargo.toml` (e.g. `0.1.0` → `0.1.1`) and commit on `main` (via PR if branch protection requires it).
2. `git tag v0.1.1 && git push origin v0.1.1`
3. Watch: `gh run watch --repo matovu-farid/murmur`
4. When complete, users can install via:
   ```
   brew install --formula https://raw.githubusercontent.com/matovu-farid/murmur/main/Homebrew/murmur.rb
   ```

   Note: the conventional `brew tap ... && brew install ...` flow requires formulas to live in `Formula/` or `HomebrewFormula/` at the repo root. To enable that later, rename `Homebrew/` → `HomebrewFormula/` in a follow-up PR (also update the `Homebrew/` path references in `release.yml` and `formula-test.yml`).

Every tag push after the first real release will:
- Produce new signed + notarized tarballs
- Create a new GitHub Release
- Auto-update `Homebrew/murmur.rb` on `main` via the formula-update job (this was skipped during the `-test` dry-run, so the first real release is also the first time this job runs for real)

---

## Self-review checklist

- [x] Every spec section maps to at least one task:
  - Repo setup → Task 2, 3
  - Workflows → Tasks 7, 8, 9
  - Secrets → Task 6 (+ Task 1 for cert export)
  - Release flow → Tasks 8, 11
  - Failure handling → inline in Task 8 + Task 11 Step 3
  - Template strategy → Task 4, applied in Task 8
  - Branch protection → Task 10
  - Dry-run → Task 11
- [x] No `TBD` / `TODO` / `implement later` placeholders
- [x] All file paths absolute or relative from repo root
- [x] All code blocks complete (no `...` elisions in commands)
- [x] Signing identity string consistent across tasks (`Developer ID Application: Farid Matovu (9VL7VRY6QZ)`)
- [x] Secret names consistent across tasks (matched against Task 6 list)
- [x] Matrix target names consistent (`aarch64-apple-darwin`, `x86_64-apple-darwin`)
- [x] Required-status-check context name matches CI job matrix rendering
