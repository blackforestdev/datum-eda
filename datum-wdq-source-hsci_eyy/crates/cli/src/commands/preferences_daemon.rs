//! Host-only Global Preferences operations that must reach the owning daemon.

use super::*;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use eda_engine::preferences::{
    AuthorizeMcpPreferenceApplyV1, McpPreferenceAuthorizationResultV1, PreferenceActorV1,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct DaemonResponse {
    result: Option<serde_json::Value>,
    error: Option<DaemonError>,
}

#[derive(Deserialize)]
struct DaemonError {
    code: i32,
    message: String,
}

pub(super) fn authorize_mcp_through_daemon(
    request: AuthorizeMcpPreferenceApplyV1,
    actor: PreferenceActorV1,
) -> Result<McpPreferenceAuthorizationResultV1> {
    let socket = std::env::var_os("DATUM_ENGINE_SOCKET")
        .or_else(|| std::env::var_os("EDA_ENGINE_SOCKET"))
        .context("DATUM_ENGINE_SOCKET/EDA_ENGINE_SOCKET is not configured")?;
    authorize_mcp_at(Path::new(&socket), request, actor)
}

fn authorize_mcp_at(
    socket: &Path,
    request: AuthorizeMcpPreferenceApplyV1,
    actor: PreferenceActorV1,
) -> Result<McpPreferenceAuthorizationResultV1> {
    let mut stream = UnixStream::connect(socket)
        .with_context(|| format!("connect owning Datum daemon at {}", socket.display()))?;
    let wire = serde_json::json!({
        "jsonrpc": "2.0",
        "id": actor.invocation_id,
        "method": "preferences.authorize_mcp",
        "params": {"request": request, "actor": actor},
    });
    serde_json::to_writer(&mut stream, &wire).context("encode daemon authorization request")?;
    stream
        .write_all(b"\n")
        .context("send daemon authorization request")?;
    stream
        .flush()
        .context("flush daemon authorization request")?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .context("read daemon authorization response")?;
    let response: DaemonResponse =
        serde_json::from_str(&line).context("decode daemon authorization response")?;
    if let Some(error) = response.error {
        anyhow::bail!("daemon refusal {}: {}", error.code, error.message);
    }
    serde_json::from_value(
        response
            .result
            .context("daemon authorization response omitted result")?,
    )
    .context("decode typed daemon authorization result")
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixListener;
    use std::thread;

    use eda_engine::preferences::PreferenceActorKindV1;

    use super::*;

    #[test]
    fn host_only_request_round_trips_without_exposing_an_acceptance_handle() {
        let root = std::env::temp_dir().join(format!("datum-preferences-cli-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let socket = root.join("daemon.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let request: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "preferences.authorize_mcp");
            assert_eq!(request["params"]["actor"]["kind"], "human_cli");
            assert!(request["params"].get("handle").is_none());
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": request["id"],
                "result": {
                    "proposal_id": request["params"]["request"]["proposal_id"],
                    "authorized": true,
                    "expires_at_unix_ms": 301000,
                },
                "error": null,
            });
            let mut stream = stream;
            serde_json::to_writer(&mut stream, &response).unwrap();
            stream.write_all(b"\n").unwrap();
        });
        let proposal_id = Uuid::new_v4();
        let result = authorize_mcp_at(
            &socket,
            AuthorizeMcpPreferenceApplyV1 {
                proposal_id,
                proposal_digest: "sha256:test".to_owned(),
                originating_mcp_session: "mcp:test".to_owned(),
            },
            PreferenceActorV1 {
                kind: PreferenceActorKindV1::HumanCli,
                session_id: "cli:test".to_owned(),
                local_actor_id: "owner".to_owned(),
                invocation_id: Uuid::new_v4(),
            },
        )
        .unwrap();
        assert_eq!(result.proposal_id, proposal_id);
        assert!(result.authorized);
        let encoded = serde_json::to_value(result).unwrap();
        assert!(encoded.get("handle").is_none());
        assert!(encoded.get("acceptance_id").is_none());
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}
