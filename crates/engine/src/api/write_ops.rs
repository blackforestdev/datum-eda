use super::*;

mod assign_package_rule;
mod basic_mutations;
mod component_replacements;
mod undo_redo;

fn merge_operation_diff(target: &mut OperationDiff, diff: &OperationDiff) {
    target.created.extend(diff.created.iter().cloned());
    target.modified.extend(diff.modified.iter().cloned());
    target.deleted.extend(diff.deleted.iter().cloned());
}
