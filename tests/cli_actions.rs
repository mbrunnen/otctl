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

#[test]
fn submit_honours_wait_timeout() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(200)
            .header("content-type", "application/vnd.api+json")
            .body(r#"{"data":[{"id":"a1","type":"updateDeviceCollectionTask"}]}"#);
    });
    server.mock(|when, then| {
        when.method(GET).path("/api/actions/a1");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data":{"id":"a1","attributes":{"status":"pending"}}}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "actions",
        "submit",
        "--file",
        "-",
        "--timeout",
        "1",
    ])
    .write_stdin(
        r#"{"data":[{"type":"updateDeviceCollectionTask","attributes":{"maxAge":30,"maxRetries":5,"deviceCount":10,"timeout":93}}]}"#,
    );
    cmd.assert().failure().code(4);
}
