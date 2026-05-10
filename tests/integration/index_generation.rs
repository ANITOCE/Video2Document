use std::fs;

use crate::common::{TestEnvironment, mount_success_mocks};
use wiremock::MockServer;

#[tokio::test]
async fn only_index_regenerates_root_and_nested_indexes() {
    let server = MockServer::start().await;
    mount_success_mocks(&server).await;
    let env = TestEnvironment::new(&server.uri());
    env.write_video("CourseA/01_绪论.mp4");

    let first = video2document::run_cli(env.cli_args())
        .await
        .expect("initial run should succeed");
    assert_eq!(first.code, 0);

    fs::remove_file(env.output_dir.join("index.md")).expect("failed to remove root index");
    fs::remove_file(env.output_dir.join("CourseA/index.md"))
        .expect("failed to remove course index");

    let mut args = env.cli_args();
    args.only_index = true;
    let report = video2document::run_cli(args)
        .await
        .expect("only-index run should succeed");

    assert_eq!(report.code, 0);
    let root_index = env.read_output("index.md");
    let course_index = env.read_output("CourseA/index.md");
    assert!(root_index.contains("[CourseA](CourseA/index.md)"));
    assert!(course_index.contains("[01_绪论](01_绪论.md)"));
}
