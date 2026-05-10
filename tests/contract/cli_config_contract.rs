use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use video2document::cli::CliArgs;
use video2document::config::load_settings_from_paths;
use video2document::models::UploadMode;

#[test]
fn cli_overrides_local_and_base_config() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let base = temp.path().join("config.toml");
    let local = temp.path().join("config.local.toml");

    fs::write(
        &base,
        "[paths]\ninput_dir = \"./base-video\"\noutput_dir = \"./base-output\"\nworking_dir = \"./base-working\"\n\n[video]\nsegment_seconds = 600\npreferred_upload_mode = \"file\"\n",
    )
    .expect("failed to write base config");
    fs::write(
        &local,
        "[paths]\noutput_dir = \"./local-output\"\n\n[video]\nsegment_seconds = 120\n",
    )
    .expect("failed to write local config");

    let args = CliArgs {
        config: Some(base.clone()),
        input_dir: Some(PathBuf::from("./cli-video")),
        output_dir: None,
        working_dir: Some(PathBuf::from("./cli-working")),
        segment_seconds: Some(30),
        retry_count: Some(5),
        upload_mode: Some(UploadMode::Base64),
        only_index: true,
        force: true,
    };

    let resolved = load_settings_from_paths(Some(&base), Some(&local), &args, true)
        .expect("failed to resolve config");

    assert_eq!(
        resolved.settings.paths.input_dir,
        PathBuf::from("./cli-video")
    );
    assert_eq!(
        resolved.settings.paths.output_dir,
        PathBuf::from("./local-output")
    );
    assert_eq!(
        resolved.settings.paths.working_dir,
        PathBuf::from("./cli-working")
    );
    assert_eq!(resolved.settings.video.segment_seconds, 30);
    assert_eq!(resolved.settings.kimi.max_retries, 5);
    assert_eq!(
        resolved.settings.video.preferred_upload_mode,
        UploadMode::Base64
    );
    assert!(resolved.only_index);
    assert!(resolved.force);
}

#[test]
fn explicit_missing_config_returns_error() {
    let args = CliArgs {
        config: Some(PathBuf::from("missing.toml")),
        input_dir: None,
        output_dir: None,
        working_dir: None,
        segment_seconds: None,
        retry_count: None,
        upload_mode: None,
        only_index: false,
        force: false,
    };

    let error = load_settings_from_paths(
        Some(PathBuf::from("missing.toml").as_path()),
        None,
        &args,
        true,
    )
    .expect_err("expected explicit missing config to fail");
    assert!(error.to_string().contains("config file not found"));
}
