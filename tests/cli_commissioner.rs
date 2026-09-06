use assert_cmd::Command;
use httpmock::prelude::*;

#[test]
fn state_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/commissioner/state");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#""active""#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", &server.base_url(), "commissioner", "state"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("active"));
    m.assert();
}

#[test]
fn state_set_posts_body() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(PUT)
            .path("/node/commissioner/state")
            .body(r#""enable""#);
        then.status(200);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "state",
        "--set",
        "enable",
    ]);
    cmd.assert().success();
    m.assert();
}

#[test]
fn joiner_list_reads_endpoint() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/commissioner/joiner");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"[{"pskd":"J01NME","joinerId":"*","timeout":60000}]"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-list",
    ]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("J01NME"));
    m.assert();
}

#[test]
fn joiner_add_from_file_posts_body() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/node/commissioner/joiner")
            .json_body(serde_json::json!({"pskd":"J01NME","joinerId":"*"}));
        then.status(200);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-add",
        "--file",
        "-",
    ])
    .write_stdin(r#"{"pskd":"J01NME","joinerId":"*"}"#);
    cmd.assert().success();
    m.assert();
}

#[test]
fn joiner_add_pskd_only_defaults_to_wildcard() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/node/commissioner/joiner")
            .json_body(serde_json::json!({"pskd":"J01NME","joinerId":"*"}));
        then.status(200);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-add",
        "--pskd",
        "J01NME",
    ]);
    cmd.assert().success();
    m.assert();
}

#[test]
fn joiner_add_eui64_with_timeout_posts_body() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/node/commissioner/joiner")
            .json_body(
                serde_json::json!({"pskd":"J01NME","eui64":"0123456789abcdef","timeout":120}),
            );
        then.status(200);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-add",
        "--pskd",
        "J01NME",
        "--eui64",
        "0123456789abcdef",
        "--timeout",
        "120",
    ]);
    cmd.assert().success();
    m.assert();
}

#[test]
fn joiner_add_discerner_posts_body() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/node/commissioner/joiner")
            .json_body(serde_json::json!({"pskd":"J01NME","discerner":"0xabc/12"}));
        then.status(200);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-add",
        "--pskd",
        "J01NME",
        "--discerner",
        "0xabc/12",
    ]);
    cmd.assert().success();
    m.assert();
}

#[test]
fn joiner_add_without_pskd_or_file_is_a_usage_error() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["commissioner", "joiner-add"]);
    cmd.assert().failure().code(2);
}

#[test]
fn joiner_add_file_conflicts_with_pskd() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "commissioner",
        "joiner-add",
        "--file",
        "j.json",
        "--pskd",
        "J01NME",
    ]);
    cmd.assert().failure().code(2);
}

#[test]
fn joiner_add_eui64_conflicts_with_discerner() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "commissioner",
        "joiner-add",
        "--pskd",
        "J01NME",
        "--eui64",
        "0123456789abcdef",
        "--discerner",
        "0xabc/12",
    ]);
    cmd.assert().failure().code(2);
}

#[test]
fn joiner_add_409_exits_3() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(POST).path("/node/commissioner/joiner");
        then.status(409)
            .header("content-type", "application/json")
            .body(r#"{"errors":[{"title":"Commissioner not active"}]}"#);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-add",
        "--pskd",
        "J01NME",
    ]);
    cmd.assert().failure().code(3);
    m.assert();
}

#[test]
fn joiner_remove_sends_id_in_body() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(DELETE)
            .path("/node/commissioner/joiner")
            .body(r#""0xabc/12""#);
        then.status(200);
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "joiner-remove",
        "0xabc/12",
    ]);
    cmd.assert().success();
    m.assert();
}

fn action_response(id: &str, status: &str) -> String {
    format!(
        r#"{{"data":{{"id":"{id}","type":"addThreadDeviceTask","attributes":{{"status":"{status}"}}}}}}"#
    )
}

#[test]
fn enroll_submits_task_and_waits_for_completion() {
    let server = MockServer::start();
    let submit = server.mock(|when, then| {
        when.method(POST).path("/api/actions").json_body(
            serde_json::json!({"data":[{"type":"addThreadDeviceTask","attributes":{
                "pskd":"J01NME","eui":"fedcba9876543210","timeout":900}}]}),
        );
        then.status(200)
            .header("content-type", "application/vnd.api+json")
            .body(r#"{"data":[{"id":"a1","type":"addThreadDeviceTask"}]}"#);
    });
    let poll = server.mock(|when, then| {
        when.method(GET).path("/api/actions/a1");
        then.status(200)
            .header("content-type", "application/json")
            .body(action_response("a1", "completed"));
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "enroll",
        "--pskd",
        "J01NME",
        "--eui",
        "fedcba9876543210",
    ]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("completed"));
    submit.assert();
    poll.assert();
}

#[test]
fn enroll_without_target_uses_wildcard_joiner_id() {
    let server = MockServer::start();
    let submit = server.mock(|when, then| {
        when.method(POST).path("/api/actions").json_body(
            serde_json::json!({"data":[{"type":"addThreadDeviceTask","attributes":{
                "pskd":"J01NME","joinerId":"*","timeout":900}}]}),
        );
        then.status(200)
            .header("content-type", "application/vnd.api+json")
            .body(r#"{"data":[{"id":"a1","type":"addThreadDeviceTask"}]}"#);
    });
    server.mock(|when, then| {
        when.method(GET).path("/api/actions/a1");
        then.status(200)
            .header("content-type", "application/json")
            .body(action_response("a1", "completed"));
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "enroll",
        "--pskd",
        "J01NME",
    ]);
    cmd.assert().success();
    submit.assert();
}

#[test]
fn enroll_discerner_and_timeout_reach_the_task() {
    let server = MockServer::start();
    let submit = server.mock(|when, then| {
        when.method(POST).path("/api/actions").json_body(
            serde_json::json!({"data":[{"type":"addThreadDeviceTask","attributes":{
                "pskd":"J01NME","discerner":"0xabc/12","timeout":120}}]}),
        );
        then.status(200)
            .header("content-type", "application/vnd.api+json")
            .body(r#"{"data":[{"id":"a1","type":"addThreadDeviceTask"}]}"#);
    });
    server.mock(|when, then| {
        when.method(GET).path("/api/actions/a1");
        then.status(200)
            .header("content-type", "application/json")
            .body(action_response("a1", "completed"));
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "enroll",
        "--pskd",
        "J01NME",
        "--discerner",
        "0xabc/12",
        "--timeout",
        "120",
    ]);
    cmd.assert().success();
    submit.assert();
}

#[test]
fn enroll_no_wait_prints_action_id() {
    let server = MockServer::start();
    let submit = server.mock(|when, then| {
        when.method(POST).path("/api/actions");
        then.status(200)
            .header("content-type", "application/vnd.api+json")
            .body(r#"{"data":[{"id":"a1","type":"addThreadDeviceTask"}]}"#);
    });
    let poll = server.mock(|when, then| {
        when.method(GET).path("/api/actions/a1");
        then.status(200).body(action_response("a1", "completed"));
    });
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "--base-url",
        &server.base_url(),
        "commissioner",
        "enroll",
        "--pskd",
        "J01NME",
        "--no-wait",
    ]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("a1"));
    submit.assert();
    poll.assert_hits(0);
}

#[test]
fn enroll_eui_conflicts_with_discerner() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args([
        "commissioner",
        "enroll",
        "--pskd",
        "J01NME",
        "--eui",
        "fedcba9876543210",
        "--discerner",
        "0xabc/12",
    ]);
    cmd.assert().failure().code(2);
}

#[test]
fn enroll_requires_pskd() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["commissioner", "enroll"]);
    cmd.assert().failure().code(2);
}
