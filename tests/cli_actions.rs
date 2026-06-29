use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn actions_submit_no_wait_prints_id() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(200).header("content-type", "application/json").body(
            r#"{"data":[{"id":"abc","type":"getNetworkDiagnosticTask","attributes":{"status":"pending"}}]}"#,
        );
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "actions",
        "submit",
        "--no-wait",
        "--file",
        "-",
    ]);
    cmd.write_stdin(r#"{"data":[]}"#);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("abc"));
    m.assert();
}
