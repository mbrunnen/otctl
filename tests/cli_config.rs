use assert_cmd::Command;
use httpmock::prelude::*;
use std::fs;

#[test]
fn config_init_writes_base_url_from_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.env("XDG_CONFIG_HOME", dir.path());
    cmd.env_remove("OTBR_URL");
    cmd.args(["config", "init"]);
    cmd.write_stdin("http://example.test\n");
    cmd.assert().success();

    let contents = fs::read_to_string(dir.path().join("otctl").join("config.toml")).unwrap();
    assert!(contents.contains("http://example.test"));
}

#[test]
fn config_init_keeps_existing_value_on_empty_input() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("otctl");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "base_url = \"http://old.example\"\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.env("XDG_CONFIG_HOME", dir.path());
    cmd.env_remove("OTBR_URL");
    cmd.args(["config", "init"]);
    cmd.write_stdin("\n");
    cmd.assert().success();

    let contents = fs::read_to_string(config_dir.join("config.toml")).unwrap();
    assert!(contents.contains("http://old.example"));
}

#[test]
fn config_file_base_url_used_when_no_flag_or_env() {
    let server = MockServer::start();
    let m = server.mock(|when, then| {
        when.method(GET).path("/node/rloc16");
        then.status(200)
            .header("content-type", "application/json")
            .body("17408");
    });

    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("otctl");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        format!("base_url = \"{}\"\n", server.base_url()),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.env("XDG_CONFIG_HOME", dir.path());
    cmd.env_remove("OTBR_URL");
    cmd.args(["node", "rloc16"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("17408"));
    m.assert();
}
