use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn node_rloc16_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/rloc16");
        then.status(200)
            .header("content-type", "application/json")
            .body("17408");
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "node", "rloc16"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("17408"));
    m.assert();
}
