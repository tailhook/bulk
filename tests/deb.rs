extern crate tempfile;
extern crate assert_cli;

use std::fs::{create_dir, write};

#[test]
fn long_file() {
    let dir = tempfile::tempdir().unwrap();
    let pkg = dir.path().join("pkg");
    create_dir(&pkg).unwrap();
    let dist = dir.path().join("dist");
    create_dir(&dist).unwrap();
    write(pkg.join(
        "0123456789012345678901234567890123456789012345678901234567890\
         12345678901234567890123456789012345678901234567890123456789"),
        "hello-long-file").unwrap();
    assert_cli::Assert::main_binary()
        .with_args(&["pack", "--dir"])
        .with_args(&[&pkg])
        .with_args(&["--dest-dir"])
        .with_args(&[&dist])
        .stderr().satisfies(|x| x.len() == 0, "bad output")
        .unwrap();
}
