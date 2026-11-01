Datum bundled native symbol library

Source: KiCad symbols Debian package 9.0.8-1~bpo13+1, installed
/usr/share/kicad/symbols. Upstream: https://gitlab.com/kicad/libraries/kicad-symbols
Copyright: KiCad Community. Upstream CC-BY-SA 4.0 and the electronic-design
exception are reproduced in COPYRIGHT.txt, copied unchanged from the installed
package. Its separate Debian-packaging notice is retained for provenance.
The converted library data retains the upstream license; this does not change
the license of Datum's Rust implementation. Conversion changes the storage
format and uses the native geometry/display conventions noted per symbol.

Coverage: 223 source libraries, 22,583 named source symbols. 17,527 symbols have
native Unit/Symbol records; 5,056 have explicit conversion refusals in the index.
This is complete source inventory, not complete conversion or standards approval.
Exact per-file source hashes and source format versions are in catalog.json.
Read the distributed upstream license and design exception with:
  datum-eda pool license

catalog.json uses the public kicad_library::bundle_format schema. Successful
entries identify native records by global byte offset, byte length and SHA256.
symbols-00.jsonl.part through symbols-03.jsonl.part concatenate, in that order,
to one UTF-8 JSONL payload. Fixed 48,000,000-byte boundaries may bisect records.
Record lengths exclude the following newline. The complete payload is
144,587,321 bytes; SHA256:
08efc7a52becd9eedb782814e6ddcc32b97023096e11225b79bcb86123ec3396

The application embeds these immutable native assets; it does not need KiCad,
network access, or the source files. Selected native records are snapshotted into
the project's journaled library. The maintenance converter is not run at startup
or during ordinary builds. IDs use logical kicad/<filename> source identities;
release versions and original file hashes remain provenance rather than IDs.

Regenerate from the exact pinned source package into a new scratch directory:
python3 scripts/run_cargo_guarded.py --workload proof -- cargo run --release -p eda-engine --example export_native_symbol_library -- --source /usr/share/kicad/symbols --source-version 9.0.8-1~bpo13+1 --report NEW_DIRECTORY/catalog.json --native-output NEW_DIRECTORY/native.jsonl
Omit --native-output for report-only measurement. Both output paths must be new.
Split the payload with Python's standard library, preserving the index offsets:
  from pathlib import Path
  source = Path('NEW_DIRECTORY')
  data = (source / 'native.jsonl').read_bytes()
  for index, offset in enumerate(range(0, len(data), 48_000_000)):
      (source / f'symbols-{index:02}.jsonl.part').write_bytes(data[offset:offset+48_000_000])
Validate each indexed record's hash, IDs, native Unit link and source provenance
before publishing replacement assets. If the chunk count changes, update the
reader's embedded chunk list. Preserve refused entries and refresh this coverage
summary; never relabel a partial conversion as complete.
