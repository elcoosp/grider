//! End-to-end tests for the `grider` binary through `assert_cmd`.

use assert_cmd::Command;
use predicates::str::contains;

fn grider() -> Command {
    Command::cargo_bin("grider").expect("binary builds")
}

fn fixture() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/small.png")
}

#[test]
fn help_and_version_work() {
    grider()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage:"));
    grider()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_argument_fails() {
    grider()
        .assert()
        .failure()
        .stderr(contains("required arguments were not provided"));
}

#[test]
fn missing_image_file_fails() {
    grider()
        .arg("does-not-exist.png")
        .assert()
        .failure()
        .stderr(contains("Failed to open image"));
}

#[test]
fn corrupt_image_file_fails() {
    let dir = tempfile_dir();
    let path = dir.join("corrupt.png");
    std::fs::write(&path, b"this is not an image").unwrap();
    grider()
        .arg(&path)
        .assert()
        .failure()
        .stderr(contains("Failed to open image"));
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(feature = "drawing")]
#[test]
fn overlay_is_written_next_to_input_by_default() {
    let dir = tempfile_dir();
    let input = dir.join("input.png");
    std::fs::copy(fixture(), &input).unwrap();

    grider()
        .arg(&input)
        .assert()
        .success()
        .stdout(contains("Extracted grid in"))
        .stdout(contains("Grid { rows:"))
        .stdout(contains("Full rows:"))
        .stdout(contains("Saving overlay to"));
    assert!(dir.join("input_with_grid.png").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(feature = "drawing")]
#[test]
fn custom_flags_change_the_output_path_and_summary() {
    let dir = tempfile_dir();
    let output = dir.join("overlay.png");

    grider()
        .args([
            fixture(),
            "--block-size",
            "5",
            "--delta",
            "0",
            "--merge-ratio",
            "0.5",
            "--no-parallel",
            "--output",
        ])
        .arg(&output)
        .assert()
        .success()
        .stdout(contains("Saving overlay to"));
    assert!(output.exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(feature = "drawing")]
#[test]
fn quiet_suppresses_summaries() {
    let dir = tempfile_dir();
    let input = dir.join("input.png");
    std::fs::copy(fixture(), &input).unwrap();

    let assertion = grider().arg(&input).arg("--quiet").assert().success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains("Extracted grid in"));
    assert!(dir.join("input_with_grid.png").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(feature = "drawing")]
#[test]
fn unwritable_output_path_fails_with_context() {
    // The parent directory does not exist, so saving the overlay must fail
    // with the wrapped "Failed to save overlay image" error.
    let output = tempfile_dir().join("missing-dir").join("overlay.png");
    grider()
        .args([fixture(), "--output"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(contains("Failed to save overlay image"));
}

#[cfg(feature = "serde")]
#[test]
fn json_output_prints_the_grid() {
    let dir = tempfile_dir();
    let input = dir.join("input.png");
    std::fs::copy(fixture(), &input).unwrap();

    let output = String::from_utf8(
        grider()
            .arg(&input)
            .arg("--json")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap();

    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(
        value
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .is_some()
    );
    assert!(
        value
            .get("columns")
            .and_then(serde_json::Value::as_array)
            .is_some()
    );
    // JSON mode does not write an overlay.
    #[cfg(feature = "drawing")]
    assert!(!dir.join("input_with_grid.png").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(not(feature = "serde"))]
#[test]
fn json_without_serde_feature_is_reported() {
    grider()
        .args([fixture(), "--json"])
        .assert()
        .success()
        .stderr(contains("Skipping JSON output"));
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "grider_cli_test_{}_{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("main")
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn sauvola_k_flag_implies_sauvola_binarization() {
    // Exercises the `--sauvola-k` handling path (sauvola without an explicit
    // --binarization flag).
    let dir = tempfile_dir();
    let input = dir.join("input.png");
    std::fs::copy(fixture(), &input).unwrap();

    grider()
        .args([fixture(), "--sauvola-k", "0.3", "--json"])
        .assert()
        .success();
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn binarization_flag_selects_sauvola_with_default_k() {
    // Exercises the explicit `--binarization sauvola` branch (default k).
    grider()
        .args([fixture(), "--binarization", "sauvola", "--json"])
        .assert()
        .success();
}

#[test]
fn mean_binarization_flag_is_accepted() {
    grider()
        .args([fixture(), "--binarization", "mean", "--json"])
        .assert()
        .success();
}

#[test]
fn invalid_binarization_flag_fails() {
    grider()
        .args([fixture(), "--binarization", "wilhelm"])
        .assert()
        .failure();
}
