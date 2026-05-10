use crate::common::{TestEnvironment, mount_success_mocks};
use wiremock::MockServer;

#[tokio::test]
async fn generates_one_markdown_per_video_with_required_sections() {
    let server = MockServer::start().await;
    mount_success_mocks(&server).await;
    let env = TestEnvironment::new(&server.uri());
    env.write_video("CourseA/01_绪论.mp4");
    env.write_text_file("CourseA/readme.txt", "ignore me");

    let report = video2document::run_cli(env.cli_args())
        .await
        .expect("run should succeed");

    assert_eq!(report.code, 0);
    assert_eq!(report.summary.processed, 1);
    assert_eq!(report.summary.failed, 0);
    assert!(env.output_dir.join("CourseA/01_绪论.md").exists());
    assert!(!env.output_dir.join("CourseA/readme.md").exists());
    assert!(env.output_dir.join("index.md").exists());
    assert!(env.output_dir.join("CourseA/index.md").exists());
    assert!(env.working_dir.join("CourseA/01_绪论/status.json").exists());

    let content = env.read_output("CourseA/01_绪论.md");
    assert!(content.contains("# 01_绪论"));
    assert!(content.contains("## 本讲目标"));
    assert!(content.contains("## 问答与自测"));
}
