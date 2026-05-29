# Mor-SteG

**Mor-SteG** is a tiny, noob-friendly Rust front-end for `steghide`.

It is designed for people who do **not** want to memorize commands like this:

```bash
steghide embed -cf picture.jpg -ef secret.txt -p password
```

Instead, you run:

```bash
mor-steg
```

Then pick from a plain menu.

## Goals

- No command memorization.
- No networking.
- No telemetry.
- No plugin system.
- No auto-updater.
- No dependency pile.
- Public-domain-style source code.
- Small enough to audit without needing a wizard hat.

## Current dependency policy

Mor-SteG intentionally uses **zero Rust dependencies**.

The app should rely only on:

- Rust standard library
- The locally installed `steghide` command
- The operating system

That means the dependency tree is intentionally boring:

```text
mor-steg
└── Rust standard library
    └── local steghide binary
```

This keeps the project small and reduces supply-chain risk.

## Cargo.toml

Recommended:

```toml
[package]
name = "mor-steg"
version = "0.1.0"
edition = "2021"
license = "Unlicense"
description = "A tiny noob-friendly Rust front-end for steghide."

[dependencies]
```

There are no crates listed under `[dependencies]` on purpose.

## Why no dependencies?

Dependencies are not automatically bad, but every dependency can add:

- more code to audit
- more maintainers to trust
- more transitive crates
- more vulnerability surface
- more license bookkeeping
- more build complexity

For a tiny `steghide` helper, the safest first version is a small menu-driven CLI.

## Features

Planned first version:

```text
[1] Hide a file inside an image/audio file
[2] Extract a hidden file
[3] Inspect a file
[4] Check if steghide is installed
[5] Quit
```

The app asks plain-English questions like:

```text
Cover file path:
Secret file path:
Password:
```

Then it runs `steghide` safely using `std::process::Command`.

## Security choices

Mor-SteG should follow these rules:

1. **Never use shell command strings.**

   Good:

   ```rust
   Command::new("steghide")
       .arg("embed")
       .arg("-cf")
       .arg(&cover_file)
       .arg("-ef")
       .arg(&secret_file)
       .arg("-p")
       .arg(&password);
   ```

   Avoid:

   ```rust
   Command::new("sh")
       .arg("-c")
       .arg(format!("steghide embed -cf {} -ef {}", cover_file, secret_file));
   ```

2. **Do not log passwords.**

   The app may pass the password to `steghide`, but it should never print it back to the terminal or write it to a config file.

3. **No networking.**

   Mor-SteG should not make web requests.

4. **No auto-updater.**

   Users should update it manually.

5. **No plugin system.**

   Plugins are fun. Plugins are also little trapdoors with hats.

6. **Check file paths before running.**

   The app should verify that the cover file, secret file, or stego file exists before invoking `steghide`.

7. **Keep the scope small.**

   Embed, extract, inspect, quit.

## Installing steghide

Mor-SteG does not implement steganography itself yet. It wraps the installed `steghide` program.

On Arch Linux:

```bash
sudo pacman -S steghide
```

On Debian/Ubuntu:

```bash
sudo apt install steghide
```

Then verify:

```bash
steghide --version
```

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
```

Or after building:

```bash
./target/release/mor-steg
```

## License

This project is released into the public domain under the **Unlicense**.

See [`LICENSE`](LICENSE).

## Optional future dependency policy

The default build should stay zero-dependency.

If convenience features are added later, they should be optional Cargo features.

Example:

```toml
[features]
default = []
hidden-password = ["dep:rpassword"]

[dependencies]
rpassword = { version = "7", optional = true }
```

However, the first version should not need this. Password input may be visible in the terminal, with a warning to the user.

Boring first. Fancy later.
