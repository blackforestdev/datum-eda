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
    assert_eq!(journal["count"], 2);
    assert_eq!(journal["transactions"][0]["operations"], 1);
    assert_eq!(journal["transactions"][1]["reason"], reason);
    assert_eq!(journal["transactions"][1]["operations"], operations);
}
