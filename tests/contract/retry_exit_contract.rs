use wiremock::MockServer;

use crate::common::{TestEnvironment, mount_failing_chat, mount_success_mocks};

#[tokio::test]
async fn partial_failure_returns_exit_code_2() {
    let server = MockServer::start().await;
    mount_failing_chat(&server).await;
    let env = TestEnvironment::new(&server.uri());
    env.write_video("CourseA/01_绪论.mp4");

    let report = video2document::run_cli(env.cli_args())
        .await
        .expect("run should complete with partial failure code");

    assert_eq!(report.code, 2);
    assert_eq!(report.summary.failed, 1);
}

#[tokio::test]
async fn force_bypasses_cache_and_reprocesses_video() {
    let first_server = MockServer::start().await;
    mount_success_mocks(&first_server).await;
    let env = TestEnvironment::new(&first_server.uri());
    env.write_video("CourseA/01_绪论.mp4");

    let first_report = video2document::run_cli(env.cli_args())
        .await
        .expect("initial run should succeed");
    assert_eq!(first_report.code, 0);

    let failing_server = MockServer::start().await;
    mount_failing_chat(&failing_server).await;
    env.rewrite_base_url(&failing_server.uri());

    let mut args = env.cli_args();
    args.force = true;
    let report = video2document::run_cli(args)
        .await
        .expect("forced run should still complete with a partial failure code");

    assert_eq!(report.code, 2);
    assert_eq!(report.summary.failed, 1);
}
