# Datum TerminalCore Unicode data

This directory contains the exact Unicode Consortium standards-data inputs for
Datum TerminalCore DTC-P11. The governed version is Unicode 17.0.0 and Emoji
17.0. These files are data, not executable code or a Cargo dependency.

Exact sources retrieved on 2026-08-19:

| Checked-in path | Unicode Consortium source |
|---|---|
| `17.0.0/DerivedCoreProperties.txt` | `https://www.unicode.org/Public/17.0.0/ucd/DerivedCoreProperties.txt` |
| `17.0.0/EastAsianWidth.txt` | `https://www.unicode.org/Public/17.0.0/ucd/EastAsianWidth.txt` |
| `17.0.0/GraphemeBreakProperty.txt` | `https://www.unicode.org/Public/17.0.0/ucd/auxiliary/GraphemeBreakProperty.txt` |
| `17.0.0/GraphemeBreakTest.txt` | `https://www.unicode.org/Public/17.0.0/ucd/auxiliary/GraphemeBreakTest.txt` |
| `17.0.0/emoji-data.txt` | `https://www.unicode.org/Public/17.0.0/ucd/emoji/emoji-data.txt` |
| `17.0.0/emoji-variation-sequences.txt` | `https://www.unicode.org/Public/17.0.0/ucd/emoji/emoji-variation-sequences.txt` |
| `17.0.0/emoji-sequences.txt` | `https://www.unicode.org/Public/17.0.0/emoji/emoji-sequences.txt` |
| `17.0.0/emoji-zwj-sequences.txt` | `https://www.unicode.org/Public/17.0.0/emoji/emoji-zwj-sequences.txt` |
| `LICENSE.txt` | `https://www.unicode.org/license.txt` |

The retained `LICENSE.txt` is the Unicode License v3 (`Unicode-3.0`). Generation
is manual and offline: run `python3 scripts/generate_terminal_unicode.py` after
verifying the checked-in inputs, or add `--check` to prove the generated Rust
tables are current. Builds and runtime never access the network.

`SHA256SUMS` pins every input and the retained license. Updating any version,
file, checksum, or policy requires a separately governed compatibility review.
