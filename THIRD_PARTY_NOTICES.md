# Third-Party Notices

This file is a practical third-party license notice for MorSteg users and packagers.

It is not legal advice.

## MorSteg's own code

MorSteg's original source code, documentation, packaging scripts, and project
files are released under the MIT License.

See:

```text
LICENSE
```

The MIT License applies only to MorSteg's own project files. It does not
relicense third-party projects, crates, external tools, package-manager
dependencies, operating-system components, or build tools.

## Summary

| Component | How MorSteg uses it | License |
| --- | --- | --- |
| MorSteg original project files | This repo's source/docs/packaging | MIT |
| `tempfile` | Rust crate dependency for safer temporary workspaces | MIT OR Apache-2.0 |
| `age` | External encryption command invoked by MorSteg | BSD-3-Clause |
| `steghide` | External steganography command invoked by MorSteg | GNU GPL; commonly GPL-2.0/GPLv2 in package metadata |
| `bubblewrap` | Optional external sandbox command for `mor-steg-sandboxed` | LGPL-2.0-or-later |

## Important boundary

MorSteg does not vendor, copy, or relicense `age`, `steghide`, or
`bubblewrap`.

MorSteg runs those tools as separate local commands.

MorSteg's release packages may declare those tools as dependencies,
recommendations, or optional dependencies. Their own packages should provide
their own license files.

## Why MIT?

MorSteg previously used the Unlicense. This project now uses the MIT License
because it is a very permissive, widely recognized software license and is one
of the license choices available for the `tempfile` crate.

MIT is permissive, but it is not the same thing as placing every third-party
component in the public domain. Third-party components remain under their own
licenses.

## MorSteg

- Component: MorSteg original project files
- License: MIT
- Applies to: MorSteg source code, docs, packaging scripts, and project files
- Does not apply to: third-party projects or external command-line tools

Practical meaning:

- Commercial use is allowed.
- Modification is allowed.
- Distribution is allowed.
- Private use is allowed.
- The copyright notice and license text should be preserved.

## tempfile

- Project: `tempfile`
- Use in MorSteg: Rust crate dependency for safer temporary workspaces
- License: `MIT OR Apache-2.0`

Practical meaning:

- MorSteg can use `tempfile` under either the MIT License or the Apache License 2.0.
- Distributors should preserve applicable copyright and license notices.
- Apache-2.0 includes additional patent-license and notice terms if that license option is chosen.

Full license references:

- MIT License: https://opensource.org/license/mit
- Apache License 2.0: https://www.apache.org/licenses/LICENSE-2.0

## age

- Project: `age`
- Use in MorSteg: external encryption command
- License: `BSD-3-Clause`

Practical meaning:

- Redistribution is allowed in source or binary form.
- Copyright notices, license text, and disclaimers must be preserved.
- The names of the copyright holder or contributors may not be used to endorse
  derived products without permission.

Full license reference:

- BSD 3-Clause: https://opensource.org/license/bsd-3-clause

## steghide

- Project: `steghide`
- Use in MorSteg: external steganography command
- License: GNU GPL; package metadata commonly lists GPLv2/GPL-2.0

Practical meaning:

- If you distribute `steghide` itself, GPL terms apply.
- GPL generally requires preserving GPL license terms and providing source-code access
  for the GPL-covered program when distributing it.
- MorSteg does not copy or link steghide. It invokes a locally installed
  `steghide` command.

Full license reference:

- GPL-2.0: https://www.gnu.org/licenses/old-licenses/gpl-2.0.html

## bubblewrap

- Project: `bubblewrap`
- Use in MorSteg: optional external sandbox command for `mor-steg-sandboxed`
- License: commonly packaged as `LGPL-2.0-or-later`

Practical meaning:

- If you distribute `bubblewrap` itself or a modified copy, LGPL terms apply.
- MorSteg does not copy or link bubblewrap. It invokes a locally installed
  `bwrap` command.

Full license reference:

- LGPL-2.0: https://www.gnu.org/licenses/old-licenses/lgpl-2.0.html

## Release packaging note

MorSteg release packages should include at least:

```text
LICENSE
THIRD_PARTY_NOTICES.md
README.md
```

For binary releases, packagers may also generate a complete Rust dependency
license report from `Cargo.lock` with tools such as `cargo-about` or `cargo-deny`.

This notice covers the main third-party projects MorSteg directly uses or invokes.
It may not be a complete legal bill of materials for every transitive crate in a
compiled binary.
