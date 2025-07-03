use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn integration_test_rule006() {
    let mut cmd = Command::cargo_bin("supa-mdx-lint").unwrap();
    cmd.arg("tests/rule006/rule006.mdx")
        .arg("--config")
        .arg("tests/rule006/supa-mdx-lint.config.toml");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("1 error"))
        .stdout(predicate::str::contains("Rule006AdmonitionLineSeparation"));
}
