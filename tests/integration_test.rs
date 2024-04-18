use assert_cmd::prelude::*;
use flate2::read::GzDecoder;
use indoc::indoc;
use std::fs::File;
use std::process::{Command, Stdio};
use tar::Archive;
use tempdir::TempDir;

#[test]
fn test_add() {
    let tar_gz = File::open("tests/fixtures/example_file_system.tar.gz").unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    let tmp_dir = TempDir::new("").unwrap();
    archive.unpack(&tmp_dir).unwrap();

    let tmp_dir_path = tmp_dir.into_path();

    let config_file_str = indoc! {r#"
        # Ignore download directory
        [[rules]]
        type = "path"
        path = "~/Downloads"

        # Ignore some volumne
        [[rules]]
        type = "path"
        path = "/Volumes/My SSD"

        # Ignore all git repositories
        [[rules]]
        type = "git_repositories"
        search = ["~/Desktop"]
    "#};

    let config_file_path = &tmp_dir_path.join("example_config.toml");
    std::fs::write(&config_file_path, config_file_str).unwrap();

    println!("Executing {}", env!("CARGO_PKG_NAME"));
    println!("---");

    let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--debug",
            "--prefix-dir",
            &tmp_dir_path.to_str().unwrap(),
            "--home-dir",
            &tmp_dir_path.join("Users/bob").to_str().unwrap(),
            "apply",
            "--config",
            &config_file_path.to_str().unwrap(),
            "--preview",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to spawn {}", env!("CARGO_PKG_NAME")));

    let output = cmd
        .wait_with_output()
        .expect(&format!("Failed to wait for {}", env!("CARGO_PKG_NAME")));

    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();

    print!("{stderr}");

    println!("---");

    assert!(stderr.contains("Preview mode is active, no changes will be applied"));
    assert!(stderr.contains(&format!(
        "Using config file '{}'",
        config_file_path.display()
    )));
    assert!(stderr.contains("Found 1 git repositories"));

    assert!(stderr.contains(&format!(
        "+ {}",
        std::fs::canonicalize(tmp_dir_path.join("Volumes/My SSD"))
            .unwrap()
            .display()
    )));
    assert!(stderr.contains(&format!(
        "+ {}",
        std::fs::canonicalize(tmp_dir_path.join("Users/bob/Downloads"))
            .unwrap()
            .display()
    )));
    assert!(stderr.contains(&format!(
        "+ {}",
        std::fs::canonicalize(tmp_dir_path.join("Users/bob/Desktop/project_a"))
            .unwrap()
            .display()
    )));
}
