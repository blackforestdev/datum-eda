"""Bootstrap delegation policy shared by the private-writer gate and its tests."""
from pathlib import Path


def check_project_bootstrap(relative: Path, text: str) -> list[str]:
    failures = []
    # The CLI delegates to the engine's units-aware genesis entry; it owns no
    # root serializer. Require the actual call, spec and canonical import.
    for pattern in (
        "bootstrap_native_project_with_units(",
        "GenesisSpec {",
        "use eda_engine::api::native_write::genesis::",
    ):
        if pattern not in text:
            failures.append(
                f"{relative}: missing expected project bootstrap pattern `{pattern}`"
            )
    write_count = text.count("write_canonical_json(")
    if write_count:
        failures.append(
            f"{relative}: expected 0 project bootstrap write_canonical_json calls "
            f"(genesis is engine-owned), found {write_count}"
        )
    for line, content in enumerate(text.splitlines(), start=1):
        if "std::fs::write(" in content:
            failures.append(
                f"{relative}:{line}: forbidden project bootstrap writer "
                f"`std::fs::write(`: {content.strip()}"
            )
    return failures
