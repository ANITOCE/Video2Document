use crate::common::{TestEnvironment, mount_success_mocks};
use wiremock::MockServer;

#[tokio::test]
async fn second_run_reuses_existing_successful_outputs() {
    let server = MockServer::start().await;
    mount_success_mocks(&server).await;
    let env = TestEnvironment::new(&server.uri());
    env.write_video("CourseA/01_绪论.mp4");

    let first = video2document::run_cli(env.cli_args())
        .await
        .expect("initial run should succeed");
    assert_eq!(first.summary.processed, 1);

    drop(server);
    let second = video2document::run_cli(env.cli_args())
        .await
        .expect("cached rerun should succeed without upstream access");

    assert_eq!(second.code, 0);
    assert_eq!(second.summary.reused, 1);
    assert_eq!(second.summary.processed, 0);
}
