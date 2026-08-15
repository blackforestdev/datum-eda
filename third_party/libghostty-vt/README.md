# libghostty-vt dependency pin

Datum uses Ghostty's terminal-state library behind its private `TerminalCore`
adapter. The source is not vendored here. `lock.json` pins the exact upstream
commit, source archive, Zig compiler, build options, complete feature set, and
retained MIT notice.

Validate the checked-in pin without network access:

```bash
python3 scripts/build_libghostty_vt.py check
```

Fetch the checksum-pinned inputs and reproduce the verified x86_64 Linux build:

```bash
python3 scripts/build_libghostty_vt.py build
```

The default cache and install prefix live below `target/libghostty-vt/` and are
gitignored. The build compiles and runs a strict C program against the installed
shared library; the adapter corpus and upstream test invocation are separate
CORE-04/CORE-05 gates rather than a hidden dependency-fetch side effect. A
different source commit, Zig version, feature set, optimization mode, or platform
is a compatibility change, not an incidental dependency refresh.
