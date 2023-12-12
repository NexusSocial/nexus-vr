use std::process::{Command, Stdio};

pub fn linux() {
    linux_check_dep();
    linux_build();
}

fn linux_check_dep() {

}

fn linux_build() {
    Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("social-client")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .spawn()
        .unwrap()
        .wait().unwrap();
}