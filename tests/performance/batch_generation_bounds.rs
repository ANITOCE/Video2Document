use crate::common::{TestEnvironment, mount_success_mocks};
use wiremock::MockServer;

#[tokio::test]
async fn segment_planning_does_not_issue_extra_requests() {
    let server = MockServer::start().await;
    mount_success_mocks(&server).await;
    let env = TestEnvironment::new(&server.uri());
    env.write_video("CourseA/01_绪论.mp4");

    let mut args = env.cli_args();
    args.segment_seconds = Some(5);
    let report = video2document::run_cli(args)
        .await
        .expect("run should succeed");
    assert_eq!(report.code, 0);

    let requests = server
        .received_requests()
        .await
        .expect("failed to read requests");
    let uploads = requests
        .iter()
        .filter(|request| request.url.path() == "/v1/files")
        .count();
    let chats = requests
        .iter()
        .filter(|request| request.url.path() == "/v1/chat/completions")
        .count();
    assert_eq!(uploads, 3);
    assert_eq!(chats, 3);
}
