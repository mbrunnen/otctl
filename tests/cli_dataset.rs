use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn dataset_active_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/dataset/active");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"NetworkName":"OpenThread"}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "dataset", "active"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("OpenThread"));
    m.assert();
}

#[test]
fn dataset_active_empty_204_is_ok() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/dataset/active");
        then.status(204);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "dataset", "active"]);
    cmd.assert().success();
    m.assert();
}

#[test]
fn dataset_active_400_exits_3() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/dataset/active");
        then.status(400)
            .header("content-type", "application/json")
            .body(r#"{"errors":[{"title":"Bad Request"}]}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "dataset", "active"]);
    cmd.assert().failure().code(3);
    m.assert();
}
