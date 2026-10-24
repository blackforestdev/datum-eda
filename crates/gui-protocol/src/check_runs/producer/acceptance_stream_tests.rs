//! Transport negatives use a finite first-party child, not product ERC evidence.
use super::*;
use std::io::Cursor;

fn request() -> CheckProducerRequest {
    CheckProducerRequest {
        job_id: Uuid::from_u128(1),
        project_id: Uuid::from_u128(2),
        model_revision: "c".repeat(64),
        project_root: "/transport-control".into(),
    }
}
fn wire(request: &CheckProducerRequest) -> Vec<u8> {
    format!(
        "{ACCEPTED_PREFIX}{}\n",
        serde_json::to_string(&request.acceptance()).unwrap()
    )
    .into_bytes()
}
struct OneByte<R>(R);
impl<R: Read> Read for OneByte<R> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(&mut output[..1])
    }
}

#[test]
fn acknowledgement_framing_is_bounded_and_survives_arbitrary_pipe_boundaries() {
    let request = request();
    let event = wire(&request);
    let mut input = vec![b'x'; event.len() * 64];
    input.push(b'\n');
    input.extend(&event);
    let mut accepted = Vec::new();
    let receipt = read(OneByte(Cursor::new(&input)), &request, |value| {
        accepted.push(value)
    })
    .unwrap();
    assert_eq!(accepted, vec![request.acceptance()]);
    assert!(receipt.error.is_none());
    assert_eq!(receipt.stderr_bytes, input.len() as u64);
    for candidate in [
        [
            ACCEPTED_PREFIX.as_bytes(),
            &vec![b'x'; event.len() * 64],
            b"\n",
        ]
        .concat(),
        event[..event.len() - 1].to_vec(),
        [ACCEPTED_PREFIX.as_bytes(), &[0xff], b"\n"].concat(),
        [ACCEPTED_PREFIX.as_bytes(), b"{}\n"].concat(),
    ] {
        let receipt = read(OneByte(Cursor::new(&candidate)), &request, |_| {
            panic!("Invalid frame acknowledged")
        })
        .unwrap();
        assert!(!receipt.accepted);
        assert!(receipt.error.is_some());
        assert_eq!(receipt.stderr_bytes, candidate.len() as u64);
    }
    let receipt = read(
        Cursor::new([&event[..], &event[..]].concat()),
        &request,
        |_| {},
    )
    .unwrap();
    assert!(receipt.accepted);
    assert!(receipt.error.unwrap().contains("Duplicate"));
    let mut foreign = request.clone();
    foreign.project_id = Uuid::from_u128(3);
    let receipt = read(Cursor::new(wire(&foreign)), &request, |_| {
        panic!("Foreign basis acknowledged")
    })
    .unwrap();
    assert!(!receipt.accepted);
    assert!(receipt.error.unwrap().contains("Foreign"));
}

#[test]
fn refused_child_drains_both_pipes_and_is_reaped_before_the_worker_returns() {
    let request = request();
    let event = wire(&request);
    let cases = [
        (
            "oversized",
            [
                ACCEPTED_PREFIX.as_bytes(),
                &vec![b'x'; event.len() * 64],
                b"\n",
            ]
            .concat(),
        ),
        ("no-delimiter", event[..event.len() - 1].to_vec()),
        (
            "invalid-utf8",
            [ACCEPTED_PREFIX.as_bytes(), &[0xff], b"\n"].concat(),
        ),
        ("duplicate", [&event[..], &event[..]].concat()),
    ];
    for (name, bytes) in cases {
        // Fill stdout beyond ordinary pipe capacity before emitting stderr.
        // The production worker must concurrently drain it even on rejection.
        let script = format!(
            "import os\nos.write(1,b'x'*{})\nb={}\nwhile b:\n n=os.write(2,b)\n b=b[n:]\n",
            event.len() * 8192,
            format!("bytes({bytes:?})")
        );
        let cli = vec!["python3".into(), "-c".into(), script];
        let mut count = 0;
        let result = super::super::worker::run_with_cli(&cli, &request, |_| count += 1);
        assert!(result.is_err(), "{name} cannot complete a GUI job");
        assert_eq!(count, usize::from(name == "duplicate"));
    }
}

#[test]
fn failed_child_retains_bounded_diagnostics_without_changing_acceptance() {
    let request = request();
    let event = wire(&request);
    let script = format!(
        "import os\nos.write(2,b'x'*{}+b'\\n')\nos.write(2,bytes({event:?}))\nos.write(2,b'Error: canonical commit refused\\n')\nraise SystemExit(2)\n",
        DIAGNOSTIC_TAIL_BYTES * 8,
    );
    let cli = vec!["python3".into(), "-c".into(), script];
    let mut accepted = Vec::new();
    let error = super::super::worker::run_with_cli(&cli, &request, |value| accepted.push(value))
        .unwrap_err()
        .to_string();
    assert_eq!(accepted, vec![request.acceptance()]);
    assert!(error.contains("exit status: 2"));
    assert!(error.contains("Error: canonical commit refused"));
    assert!(error.len() < DIAGNOSTIC_TAIL_BYTES + 512);
    let receipt = read(
        Cursor::new(vec![b'x'; DIAGNOSTIC_TAIL_BYTES * 8]),
        &request,
        |_| panic!("diagnostics cannot acknowledge"),
    )
    .unwrap();
    assert_eq!(receipt.stderr_bytes, (DIAGNOSTIC_TAIL_BYTES * 8) as u64);
    assert_eq!(receipt.diagnostic_tail.len(), DIAGNOSTIC_TAIL_BYTES);
    assert!(!receipt.accepted);
}

#[test]
fn manual_producer_clears_inherited_agent_identity_and_preserves_commit_source() {
    const CHILD: &str = "DATUM_CHECK_PRODUCER_ENVIRONMENT_CONTROL";
    if std::env::var(CHILD).as_deref() == Ok("1") {
        // Run the real worker under an independently polluted process parent,
        // without mutating the concurrent test runner's global environment.
        assert_eq!(
            std::env::var("DATUM_AGENT_PROVENANCE").unwrap(),
            "invalid-parent-agent-json"
        );
        let request = request();
        let event = wire(&request);
        let script = format!(
            "import os\nassert 'DATUM_AGENT_PROVENANCE' not in os.environ\nassert os.environ.get('DATUM_COMMIT_SOURCE')=='tool'\nos.write(2,bytes({event:?}))\nos.write(2,b'manual_producer_environment_verified\\n')\nraise SystemExit(2)\n"
        );
        let cli = vec!["python3".into(), "-c".into(), script];
        let mut accepted = Vec::new();
        let error =
            super::super::worker::run_with_cli(&cli, &request, |value| accepted.push(value))
                .unwrap_err()
                .to_string();
        assert_eq!(accepted, vec![request.acceptance()]);
        assert!(
            error.contains("manual_producer_environment_verified"),
            "{error}"
        );
        return;
    }
    let test_name = format!(
        "{}::manual_producer_clears_inherited_agent_identity_and_preserves_commit_source",
        module_path!().split_once("::").unwrap().1
    );
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test_name, "--nocapture"])
        .env(CHILD, "1")
        .env("DATUM_AGENT_PROVENANCE", "invalid-parent-agent-json")
        .env("DATUM_COMMIT_SOURCE", "tool")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "isolated environment control failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[test]
fn candidate_output_defers_readback_and_strict_api_still_requires_it() {
    let request = request();
    let event = wire(&request);
    let output = serde_json::to_vec(&serde_json::json!({
        "project_id":request.project_id, "model_revision":request.model_revision,
        "profile_id":"erc", "persisted":true, "check_run_id":Uuid::from_u128(3),
    }))
    .unwrap();
    // Only a transport control: there is no persisted Project at this root.
    // Any early readback fails, so the candidate path must defer it to the
    // canonical snapshot gate; the existing strict API must still refuse.
    let script = format!(
        "import os,sys\nif 'show' in sys.argv: raise SystemExit(7)\nos.write(2,bytes({event:?}))\nos.write(1,bytes({output:?}))\n"
    );
    let cli = vec!["python3".into(), "-c".into(), script];
    let mut accepted = Vec::new();
    let candidate =
        super::super::worker::produce_candidate_with_cli(&cli, &request, |v| accepted.push(v))
            .unwrap();
    assert_eq!(accepted, vec![request.acceptance()]);
    assert!(request.admits_run(&candidate.run));
    assert!(super::super::worker::run_with_cli(&cli, &request, |_| {}).is_err());
}
