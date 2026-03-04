# Default Include and Exclude List

Alfred uses two built-in ignore lists with different override semantics:

1. **Always Excluded**: patterns that MUST always be excluded from indexing. **Nothing can override these rules**, not even `!` include lines in any `.alfredignore` file. This list contains items like executables and binary artifacts that are never useful for text indexing regardless of user preference.

2. **Default `.alfredignore`**: patterns applied by default but overridable. A `.alfredignore` file may include a line beginning with `!` to include a path that would otherwise be excluded by this list. For example, placing `!Cargo.lock` in a workspace `.alfredignore` re-enables indexing of `Cargo.lock`.

## Always Excluded

```gitignore
# Always excluded patterns cannot be overridden by whitelist rules.

# Alfred internals
.git/
.alfred/
.agents/

# Executables and linkable artifacts
*.a
*.dll
*.dylib
*.exe
*.lib
*.o
*.obj
*.pdb
*.rlib
*.rmeta
*.so
*.wasm
*.dSYM/

# Common binary formats (not useful for text indexing)
*.7z
*.bin
*.bz2
*.dmg
*.gz
*.iso
*.rar
*.tar
*.tgz
*.xz
*.zip
*.zst
*.pdf

# Images
*.bmp
*.gif
*.icns
*.ico
*.jpeg
*.jpg
*.png
*.svg
*.svgz
*.tif
*.tiff
*.webp
```

# Default `.alfredignore`

These patterns are applied by default but MAY be overridden by a `!` include line in a `.alfredignore` file.

```gitignore
# Rust
.analyze/
debug/
mutants.out*/
target/
*.d
**/*.rs.bk
*.crate
*.profraw
*.profdata
cargo-install-update-lock
# Cargo.lock is excluded by default, not always-excluded, because agents occasionally need
# to inspect it (for example for dependency auditing or version tracing). Workspace
# .alfredignore files can re-enable it with !Cargo.lock.
Cargo.lock
Cargo.toml.orig
crates-io-index/

# Node / JS
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
pnpm-debug.log*
.pnp.*
.yarn/
.pnpm-store/
dist/
build/
.next/
.nuxt/
.svelte-kit/
.turbo/

# Python
__pycache__/
*.py[cod]
*.pyo
.venv/
venv/
.tox/
.mypy_cache/
.pytest_cache/
.ruff_cache/
.coverage
coverage.xml
.nox/
*.egg-info/
.eggs/
pip-wheel-metadata/

# .NET
bin/
obj/
.vs/
TestResults/
packages/

# Java / Kotlin
.gradle/
.idea/
*.iml

# Go
vendor/
*.test
coverage.out

# Elixir
_build/
deps/

# Terraform
.terraform/
*.tfstate
*.tfstate.*

# ----- Aurora model generated files -----
docs/design/README-MIS-*.md
docs/design/MIS-*.md
docs/design/MIS-*/**/*
docs/design/aurora/MIS-*/Compact.json
# ----- End of Aurora model generated files -----

# Secrets, credentials, and settings
.env*
!.env.example
*.cert
*.crt
*.csr
*.der
*.jks
*.key
*.keystore
*.p12
*.pem
*.pfx
*.pvk

# Configuration Files
config.json
!config.default.json
!config.example.json
!config.schema.json

# Cache and temporary files
.cache/
.temp/
temp/
tmp/
*.tmp
*.orig
*.rej

# direnv
.direnv/

# Logs and diagnostics
*.log
*-debug.log*
*-error.log*
report.[0-9]*.[0-9]*.[0-9]*.[0-9]*.json

# Test results & coverage output
.nyc_output/
.coverage/
coverage/
*.lcov
test-results/

# Runtime data
pids/
*.pid
*.pid.lock
*.seed

# Diff/Patch files
*.diff
*.patch

# Platform-specific Cruft
## Linux
*~
.directory
.fuse_hidden*
.nfs*
.Trash-*

## Windows
$RECYCLE.BIN/
[Dd]esktop.ini
ehthumbs.db
ehthumbs_vista.db
*.lnk
*.stackdump
Thumbs.db
Thumbs.db:encryptable

## macOS
._*
.apdisk
.AppleDB
.AppleDesktop
.AppleDouble
.com.apple.timemachine.donotpresent
.DocumentRevisions-V100
.DS_Store
.fseventsd
.LSOverride
.Spotlight-V100
.TemporaryItems
.Trashes
.VolumeIcon.icns
Icon
Network Trash Folder
Temporary Items
```
