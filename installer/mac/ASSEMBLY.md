# Assemble the Mac kit

This directory contains preparation scripts and an uncompleted acceptance record,
not replacement firmware or a rebuilt executable. Source payload is pinned to
commit 097ccb96eedde986f19ece1ebb2b3b0b8d881565.

Copy this directory to `OG-riddle-Mac-RM2`. Create `payload/` inside it. Download
artifact 11663168560 from successful Actions run 38033938534 and save it there as
`riddle-rm2-original.zip`. Its SHA-256 must be:

`8e7a4d729bd1a3922b5150e988a3f0eec08abf51ea4b2e6ccf446457f93114c9`

After checking that digest, unzip its two files into the same payload directory.
Run `bash mac.sh verify` from the kit root. The script has independently pinned
ZIP and TAR hashes; do not substitute a newer workflow artifact. Read
START-HERE.md before any tablet operation. Generated bundles, private backups,
and real oracle.env files must never be committed.

The original compiled payload is deliberately not stored again in Git. Original
artifact expiry is January 8, 2027; a future rebuild requires a new provenance
record, byte checks, and appropriate revalidation rather than silently changing
this kit's pins.
