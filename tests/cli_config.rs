use assert_cmd::Command;
use httpmock::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn otctl(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.env("XDG_CONFIG_HOME", dir);
    cmd.env_remove("OTBR_URL");
    cmd.env_remove("OTBR_ROUTER");
    cmd
}

fn config_text(dir: &Path) -> String {
    fs::read_to_string(dir.join("otctl").join("config.toml")).unwrap()
}

fn add(dir: &Path, name: &str, url: &str) {
    otctl(dir)
        .args(["config", "add", name, url])
        .assert()
        .success();
}

fn rloc16_mock(server: &MockServer) -> httpmock::Mock<'_> {
    server.mock(|when, then| {
        when.method(GET).path("/node/rloc16");
        then.status(200)
            .header("content-type", "application/json")
            .body("17408");
    })
}

fn write_old_format(dir: &Path) {
    let config_dir = dir.join("otctl");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "base_url = \"http://old.invalid\"\n",
    )
    .unwrap();
}

#[test]
fn config_add_writes_router_and_default() {
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", "http://lab.test");
    let text = config_text(dir.path());
    assert!(text.contains("default = \"lab\""), "{text}");
    assert!(text.contains("url = \"http://lab.test\""), "{text}");
}

#[test]
fn config_add_existing_name_fails() {
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", "http://lab.test");
    otctl(dir.path())
        .args(["config", "add", "lab", "http://x"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
}

#[test]
fn config_list_marks_default() {
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", "http://lab.test");
    add(dir.path(), "home", "http://home.test");
    let out = otctl(dir.path())
        .args(["--json", "config", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        list,
        serde_json::json!([
            {"name": "home", "url": "http://home.test", "default": false},
            {"name": "lab", "url": "http://lab.test", "default": true},
        ])
    );
}

#[test]
fn config_default_and_remove() {
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", "http://lab.test");
    add(dir.path(), "home", "http://home.test");
    otctl(dir.path())
        .args(["config", "default", "home"])
        .assert()
        .success();
    otctl(dir.path())
        .args(["config", "remove", "lab"])
        .assert()
        .success();
    let text = config_text(dir.path());
    assert!(text.contains("default = \"home\""), "{text}");
    assert!(!text.contains("[routers.lab]"), "{text}");
}

#[test]
fn default_router_used_when_no_flag() {
    let server = MockServer::start();
    let m = rloc16_mock(&server);
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", &server.base_url());
    otctl(dir.path())
        .args(["node", "rloc16"])
        .assert()
        .success()
        .stdout(predicates::str::contains("17408"));
    m.assert();
}

#[test]
fn router_flag_selects_saved_router() {
    let a = MockServer::start();
    let b = MockServer::start();
    let mock_a = rloc16_mock(&a);
    let mock_b = rloc16_mock(&b);
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "a", &a.base_url());
    add(dir.path(), "b", &b.base_url());
    otctl(dir.path())
        .args(["--router", "b", "node", "rloc16"])
        .assert()
        .success();
    mock_b.assert();
    mock_a.assert_hits(0);
}

#[test]
fn router_env_selects_saved_router() {
    let a = MockServer::start();
    let b = MockServer::start();
    let mock_a = rloc16_mock(&a);
    let mock_b = rloc16_mock(&b);
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "a", &a.base_url());
    add(dir.path(), "b", &b.base_url());
    otctl(dir.path())
        .env("OTBR_ROUTER", "b")
        .args(["node", "rloc16"])
        .assert()
        .success();
    mock_b.assert();
    mock_a.assert_hits(0);
}

#[test]
fn unknown_router_fails_with_names() {
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", "http://lab.test");
    otctl(dir.path())
        .args(["--router", "nope", "node", "rloc16"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown router 'nope'"));
}

#[test]
fn base_url_flag_ignores_old_format_config() {
    let server = MockServer::start();
    let m = rloc16_mock(&server);
    let dir = tempfile::tempdir().unwrap();
    write_old_format(dir.path());
    otctl(dir.path())
        .args(["--base-url", &server.base_url(), "node", "rloc16"])
        .assert()
        .success();
    m.assert();
}

#[test]
fn old_format_config_fails_with_hint() {
    let dir = tempfile::tempdir().unwrap();
    write_old_format(dir.path());
    otctl(dir.path())
        .args(["node", "rloc16"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("otctl config add <name> <url>"));
}

#[test]
fn base_url_and_router_env_conflict() {
    let server = MockServer::start();
    let m = rloc16_mock(&server);
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", &server.base_url());
    otctl(dir.path())
        .env("OTBR_URL", server.base_url())
        .env("OTBR_ROUTER", "lab")
        .args(["node", "rloc16"])
        .assert()
        .code(2);
    m.assert_hits(0);
}

#[test]
fn config_list_prints_table_without_json() {
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", "http://lab.test");
    otctl(dir.path())
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("lab"))
        .stdout(predicates::str::contains("http://lab.test"))
        .stdout(predicates::str::contains("{").not());
}

#[test]
fn base_url_flag_overrides_router_env() {
    let server = MockServer::start();
    let m = rloc16_mock(&server);
    let dir = tempfile::tempdir().unwrap();
    otctl(dir.path())
        .env("OTBR_ROUTER", "lab")
        .args(["--base-url", &server.base_url(), "node", "rloc16"])
        .assert()
        .success();
    m.assert();
}

#[test]
fn router_flag_overrides_base_url_env() {
    let a = MockServer::start();
    let b = MockServer::start();
    let mock_a = rloc16_mock(&a);
    let mock_b = rloc16_mock(&b);
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "b", &b.base_url());
    otctl(dir.path())
        .env("OTBR_URL", a.base_url())
        .args(["--router", "b", "node", "rloc16"])
        .assert()
        .success();
    mock_b.assert();
    mock_a.assert_hits(0);
}

#[test]
fn base_url_and_router_flags_conflict() {
    let server = MockServer::start();
    let m = rloc16_mock(&server);
    let dir = tempfile::tempdir().unwrap();
    add(dir.path(), "lab", &server.base_url());
    otctl(dir.path())
        .args([
            "--base-url",
            &server.base_url(),
            "--router",
            "lab",
            "node",
            "rloc16",
        ])
        .assert()
        .code(2);
    m.assert_hits(0);
}
