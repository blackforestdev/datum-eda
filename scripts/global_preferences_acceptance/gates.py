"""One exact mandatory command inventory shared by capture and evidence validation."""

from pathlib import Path
import sys


def release_build():
    return ["python3", "scripts/run_cargo_guarded.py", "--workload", "proof", "--",
            "cargo", "build", "--release", "-p", "datum-eda-cli", "-p", "datum-gui-app",
            "-p", "eda-engine-daemon", "--locked", "--offline"]


def commands(root):
    root = Path(root).resolve()
    python = "python3"
    script = lambda name, *args: [python, "scripts/" + name, *args]
    guard = script("run_cargo_guarded.py", "--workload", "proof", "--")
    names = {
        "matrix": "check_global_preferences_production_matrix.py",
        "boundary": "check_global_preferences_boundary.py",
        "menu": "check_menu_model.py", "resolver": "check_resolver_raw_loads.py",
        "daemon": "check_daemon_write_parity.py", "mcp": "check_mcp_public_taxonomy.py",
        "dependencies": "check_dependency_authority.py", "cargo-resources": "check_cargo_resource_policy.py",
        "governance": "check_spec_governance.py", "parity": "check_spec_parity.py",
        "traceability": "check_evidence_traceability.py", "progress": "check_progress_coverage.py",
        "alignment": "check_alignment.py",
    }
    result = {key: [script(name)] for key, name in names.items()}
    result.update({
        "private-writer": [script("check_schematic_private_writers.py"), script("check_global_preferences_boundary.py")],
        "source-health": [script("check_source_health.py"),
                          [python, "-m", "unittest", "discover", "-s", "scripts", "-p", "test_source_health_governance.py"]],
        "project-state": [script("project_status.py", "check"), script("project_status.py", "check-render")],
        "workspace-tests": [[*guard, "cargo", "test", "--workspace", "--all-targets", "--locked", "--offline"]],
        "clippy": [[*guard, "cargo", "clippy", "--workspace", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]],
        "checker-regressions": [[python, "-m", "unittest", "discover", "-s", "scripts", "-p", "test_global_preferences_*.py"]],
    })
    return result
