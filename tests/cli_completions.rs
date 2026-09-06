use assert_cmd::Command;

#[test]
fn zsh_completion_script_is_printed() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["completions", "zsh"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("#compdef otctl"));
}

#[test]
fn bash_completion_script_is_printed() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["completions", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("_otctl"));
}

#[test]
fn completion_script_covers_subcommands() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    let out = cmd.args(["completions", "zsh"]).output().unwrap();
    let script = String::from_utf8(out.stdout).unwrap();
    for sub in ["commissioner", "enroll", "joiner-add", "dataset", "actions"] {
        assert!(script.contains(sub), "completion script lacks {sub}");
    }
}

#[test]
fn unknown_shell_is_a_usage_error() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["completions", "rc"]);
    cmd.assert().failure().code(2);
}

#[test]
fn completions_need_no_border_router() {
    let mut cmd = Command::cargo_bin("otctl").unwrap();
    cmd.args(["--base-url", "http://127.0.0.1:1", "completions", "zsh"]);
    cmd.assert().success();
}
