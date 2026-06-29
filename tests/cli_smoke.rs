use assert_cmd::Command;

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("--base-url"));
}
