use super::*;

pub(super) fn mcp_surface_result(cli: &Cli) -> Result<String> {
    let script = r#"
import pathlib
import sys
import unittest

repo = pathlib.Path(sys.argv[1])
top = repo / "mcp-server"
wanted = {
    "test_tools_call_dispatches_save",
    "test_tools_call_dispatches_delete_track",
    "test_tools_call_delete_track_changes_followup_check_report",
    "test_tools_call_dispatches_delete_component",
    "test_tools_call_delete_component_changes_followup_components_response",
    "test_tools_call_dispatches_delete_via",
    "test_tools_call_delete_via_changes_followup_net_info_response",
    "test_tools_call_dispatches_move_component",
    "test_tools_call_move_component_changes_followup_unrouted_response",
    "test_tools_call_dispatches_rotate_component",
    "test_tools_call_rotate_component_changes_followup_components_response",
    "test_tools_call_dispatches_set_design_rule",
    "test_tools_call_set_design_rule_changes_followup_design_rules_response",
    "test_tools_call_dispatches_set_value",
    "test_tools_call_set_value_changes_followup_components_response",
    "test_tools_call_dispatches_set_reference",
    "test_tools_call_set_reference_changes_followup_components_response",
    "test_tools_call_dispatches_assign_part",
    "test_tools_call_assign_part_changes_followup_net_info_response",
    "test_tools_call_assign_part_preserves_logical_nets_across_known_part_remap_response",
    "test_tools_call_dispatches_set_package",
    "test_tools_call_dispatches_set_package_with_part",
    "test_tools_call_set_package_changes_followup_net_info_response",
    "test_tools_call_set_package_preserves_logical_nets_across_known_part_remap_response",
    "test_tools_call_set_package_with_part_preserves_logical_nets_for_explicit_candidate",
    "test_tools_call_dispatches_set_net_class",
    "test_tools_call_set_net_class_changes_followup_net_info_response",
    "test_tools_call_dispatches_undo_and_redo",
}

def iter_tests(suite):
    for test in suite:
        if isinstance(test, unittest.TestSuite):
            yield from iter_tests(test)
        else:
            yield test

discovered = unittest.defaultTestLoader.discover(
    start_dir=top,
    pattern="test_*.py",
    top_level_dir=top,
)
selected = unittest.TestSuite()
found = set()
for test in iter_tests(discovered):
    name = test.id().rsplit(".", 1)[-1]
    if name in wanted:
        selected.addTest(test)
        found.add(name)

missing = sorted(wanted - found)
if missing:
    print("missing MCP parity tests:", ", ".join(missing))
    sys.exit(1)

result = unittest.TextTestRunner(verbosity=0).run(selected)
if not result.wasSuccessful():
    sys.exit(1)
print("selected MCP write-surface dispatch tests passed")
"#;

    let output = m3_write_surface_common::run_command_checked(
        Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&cli.repo_root)
            .current_dir(&cli.repo_root),
        "mcp write-surface parity probe",
    )?;

    Ok(output)
}
