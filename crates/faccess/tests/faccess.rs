#![allow(missing_docs)]
use std::io;

use faccess::faccess;

fn check_ok(result: io::Result<()>) {
    result.unwrap_or_else(|err| panic!("check_ok: {err}"));
}

fn check_err(result: io::Result<()>) {
    result.expect_err("expect error but got ok");
}

#[test]
fn runfiles() {
    let path = testing::rlocation("_main/.rustfmt.toml");

    check_ok(faccess().at(&path));
    check_ok(faccess().r_ok().at(&path));
    check_ok(faccess().real().at(&path));
    check_ok(faccess().real().r_ok().at(&path));

    // Results vary depending on spawn_strategy, so w_ok/x_ok are not checked here.
}

#[test]
fn bin_bash() {
    check_ok(faccess().r_ok().at("/bin/bash"));
    check_ok(faccess().r_ok().x_ok().at("/bin/bash"));
    check_err(faccess().w_ok().at("/bin/bash"));
}
