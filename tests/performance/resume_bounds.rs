use crate::common::{TestEnvironment, mount_success_mocks};
use wiremock::MockServer;

#[tokio::test]
async fn cached_rerun_adds_no_new_upstream_requests() {
    let server = MockServer::start().await;
    mount_success_mocks(&server).await;
    let env = TestEnvironment::new(&server.uri());
    env.write_video("CourseA/01_绪论.mp4");

    let first = video2document::run_cli(env.cli_args())
        .await
        .expect("initial run should succeed");
    assert_eq!(first.code, 0);
    let first_request_count = server
        .received_requests()
        .await
        .expect("failed to read first request set")
        .len();

    let second = video2document::run_cli(env.cli_args())
        .await
        .expect("cached rerun should succeed");
    assert_eq!(second.summary.reused, 1);

    let second_request_count = server
        .received_requests()
        .await
        .expect("failed to read second request set")
        .len();
    assert_eq!(first_request_count, second_request_count);
}
