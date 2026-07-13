use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn devices_list_refreshes_then_reads() {
    let server = MockServer::start();
    let post = server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data":[{"id":"u1","type":"updateDeviceCollectionTask","attributes":{"status":"pending"}}]}"#);
    });
    let action = server.mock(|when, then| {
        when.method(GET).path("/api/actions/u1");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"data":{"id":"u1","type":"updateDeviceCollectionTask","attributes":{"status":"stopped"}}}"#);
    });
    let list = server.mock(|when, then| {
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
    post.assert();
    action.assert_hits(1);
    list.assert();
}

#[test]
fn devices_list_no_refresh_skips_update() {
    let server = MockServer::start();
    let post = server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(200).body("{}");
    });
    let list = server.mock(|when, then| {
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
        "--no-refresh",
    ]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("data"));
    list.assert();
    post.assert_hits(0);
}
