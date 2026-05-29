# Third-Party Notices

This file is a practical third-party license notice for MorSteg users and packagers.

It is not legal advice.

## MorSteg's own code

MorSteg's original source code, documentation, packaging scripts, and project files are released under the MIT License.

See:

```text
LICENSE
```

The MIT License applies only to MorSteg's own project files. It does not relicense third-party projects, crates, external tools, package-manager dependencies, operating-system components, or build tools.

## Summary

| Component | How MorSteg uses it | License |
| --- | --- | --- |
| MorSteg original project files | This repo's source/docs/packaging | MIT |
| `tempfile` | Rust crate dependency for safer temporary workspaces | MIT OR Apache-2.0 |
| `age` | External encryption command invoked by MorSteg | BSD-3-Clause |
| `steghide` | External steganography command invoked by MorSteg | GNU GPL; commonly GPL-2.0/GPLv2 in package metadata |
| `bubblewrap` | Optional external sandbox command for `mor-steg-sandboxed` | LGPL-2.0-or-later |

## Important boundary

MorSteg does not vendor, copy, or relicense `age`, `steghide`, or `bubblewrap`.

MorSteg runs those tools as separate local commands.

MorSteg's release packages may declare those tools as dependencies, recommendations, or optional dependencies. Their own packages should provide their own license files.

## Why MIT?

MorSteg uses the MIT License because it is permissive and widely recognized for software.

MIT is permissive, but it is not the same thing as placing every third-party component in the public domain. Third-party components remain under their own licenses.

## tempfile

- Project: `tempfile`
- Use in MorSteg: Rust crate dependency for safer temporary workspaces
- License: `MIT OR Apache-2.0`

## age

- Project: `age`
- Use in MorSteg: external encryption command
- License: `BSD-3-Clause`

## steghide

- Project: `steghide`
- Use in MorSteg: external steganography command
- License: GNU GPL; package metadata commonly lists GPLv2/GPL-2.0

MorSteg does not copy or link steghide. It invokes a locally installed `steghide` command.

## bubblewrap

- Project: `bubblewrap`
- Use in MorSteg: optional external sandbox command for `mor-steg-sandboxed`
- License: commonly packaged as `LGPL-2.0-or-later`

MorSteg does not copy or link bubblewrap. It invokes a locally installed `bwrap` command.

## Release packaging note

MorSteg release packages should include at least:

```text
LICENSE
THIRD_PARTY_NOTICES.md
README.md
```

For binary releases, packagers may also generate a complete Rust dependency license report from `Cargo.lock` with tools such as `cargo-about` or `cargo-deny`.
