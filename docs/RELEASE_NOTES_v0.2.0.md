# MorSteg CLI v0.2.0

MorSteg is a noob-friendly CLI helper for age-encrypted steganography with steghide.

## What is new

- Split CLI flows into modules
- Age key helper
- Post-quantum-capable age recipient workflow
- Separate public key file generation
- Cloud-safe `.MorSteg.zip` packages
- Separate save locations for:
  - steghide/output files
  - age key files
  - cloud-safe packages
- Optional Bubblewrap sandbox launcher
- MorSteg branding cleanup

## Honest security wording

MorSteg is **post-quantum-capable** when used with a new enough `age` build and post-quantum age recipient keys.

MorSteg does **not** implement post-quantum cryptography itself.

The security model is:

```text
age       = encrypts the secret
steghide  = hides the already-encrypted payload
MorSteg   = friendly wrapper and workflow manager
```

Do not describe this release as "quantum-proof."

Better wording:

```text
Post-quantum-capable through age PQ recipient keys.
```

## Cloud-safe packages

`.MorSteg.zip` packages are for byte preservation, not encryption.

Use them when uploading steghide carrier files to cloud storage so image/audio recompression does not destroy the hidden payload.

## Release assets

Recommended assets:

```text
mor-steg-0.2.0-x86_64-unknown-linux-gnu.tar.zst
mor-steg_0.2.0_amd64.deb
mor-steg-0.2.0-1.x86_64.rpm
mor-steg-0.2.0-1-x86_64.pkg.tar.zst
mor-steg-0.2.0-nix-result.tar.zst
SHA256SUMS
SHA256SUMS.asc or SHA256SUMS.sig
```

## Verification

Download the release assets and verify checksums:

```bash
sha256sum -c SHA256SUMS
```

If a signature is provided:

```bash
gpg --verify SHA256SUMS.asc SHA256SUMS
```
