use assert_cmd::Command;

fn live_url() -> Option<String> {
    std::env::var("OTBR_TEST_URL").ok()
}

#[test]
fn live_node_state() {
    let Some(url) = live_url() else {
        eprintln!("skipping: OTBR_TEST_URL not set");
        return;
    };
    Command::cargo_bin("otctl").unwrap()
        .args(["--base-url", &url, "node", "state"])
        .assert().success();
}

#[test]
fn live_dataset_active_json() {
    let Some(url) = live_url() else {
        eprintln!("skipping: OTBR_TEST_URL not set");
        return;
    };
    Command::cargo_bin("otctl").unwrap()
        .args(["--base-url", &url, "--json", "dataset", "active"])
        .assert().success().stdout(predicates::str::contains("{"));
}
