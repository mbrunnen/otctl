use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn devices_list_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/api/devices");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data":[]}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "--json",
        "devices",
        "list",
    ]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("data"));
    m.assert();
}
