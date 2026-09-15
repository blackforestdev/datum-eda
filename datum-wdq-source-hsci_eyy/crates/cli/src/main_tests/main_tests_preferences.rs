use super::*;

#[test]
fn preferences_authorize_mcp_has_the_exact_host_only_cli_doorway() {
    let cli = Cli::try_parse_from([
        "datum-eda",
        "preferences",
        "proposal",
        "authorize-mcp",
        "--proposal-json",
        "proposal.json",
        "--mcp-session",
        "mcp:local-session",
    ])
    .unwrap();
    let Commands::Preferences {
        action:
            PreferencesCommands::Proposal {
                action:
                    PreferencesProposalCommands::AuthorizeMcp {
                        proposal_json,
                        mcp_session,
                    },
            },
    } = cli.command
    else {
        panic!("expected preferences proposal authorize-mcp")
    };
    assert_eq!(proposal_json, PathBuf::from("proposal.json"));
    assert_eq!(mcp_session, "mcp:local-session");
}
