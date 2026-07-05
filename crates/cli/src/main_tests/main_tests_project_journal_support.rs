use super::*;

pub(crate) fn assert_journal_transaction(root: &Path, reason: &str, operations: u64) {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    let journal: serde_json::Value =
        serde_json::from_str(&output).expect("journal-list JSON should parse");
    let transactions = journal["transactions"]
        .as_array()
        .expect("journal transactions should be an array");
    let latest = transactions
        .last()
        .expect("journal should contain at least one transaction");
    assert_eq!(latest["reason"], reason);
    assert_eq!(latest["operations"], operations);
}
