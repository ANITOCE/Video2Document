use std::fs;

use tempfile::TempDir;

use video2document::config::AppSettings;
use video2document::scanner::scan_videos;

#[test]
fn scanner_filters_hidden_temp_and_non_video_files() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let input = temp.path().join("Video");
    fs::create_dir_all(input.join("CourseA")).expect("failed to create input dir");
    fs::write(input.join("CourseA/01_绪论.mp4"), b"video").expect("failed to write video");
    fs::write(input.join("CourseA/readme.txt"), b"text").expect("failed to write text");
    fs::write(input.join("CourseA/~$temp.mp4"), b"temp").expect("failed to write temp video");

    let mut settings = AppSettings::default();
    settings.paths.input_dir = input;
    settings.paths.output_dir = temp.path().join("Document");
    settings.paths.working_dir = temp.path().join("Working");

    let videos = scan_videos(&settings).expect("failed to scan videos");
    assert_eq!(videos.len(), 1);
    assert_eq!(
        videos[0].relative_path,
        std::path::PathBuf::from("CourseA/01_绪论.mp4")
    );
}

#[test]
fn scanner_detects_output_collisions() {
    let temp = TempDir::new().expect("failed to create temp dir");
    let input = temp.path().join("Video");
    fs::create_dir_all(input.join("CourseA")).expect("failed to create input dir");
    fs::write(input.join("CourseA/01_绪论.mp4"), b"video").expect("failed to write first video");
    fs::write(input.join("CourseA/01_绪论.mov"), b"video").expect("failed to write second video");

    let mut settings = AppSettings::default();
    settings.paths.input_dir = input;
    settings.paths.output_dir = temp.path().join("Document");
    settings.paths.working_dir = temp.path().join("Working");

    let error = scan_videos(&settings).expect_err("expected collision detection to fail");
    assert!(error.to_string().contains("output collision"));
}
