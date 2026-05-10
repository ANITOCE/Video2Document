use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use video2document::cli::CliArgs;

static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[allow(dead_code)]
pub struct TestEnvironment {
    _guard: MutexGuard<'static, ()>,
    _root: TempDir,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub working_dir: PathBuf,
    pub config_path: PathBuf,
    old_path: Option<String>,
    old_api_key: Option<String>,
    old_ffprobe_bin: Option<String>,
    old_ffmpeg_bin: Option<String>,
}

#[allow(dead_code)]
impl TestEnvironment {
    pub fn new(base_url: &str) -> Self {
        let guard = ENV_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = TempDir::new().expect("failed to create temp dir");
        let input_dir = root.path().join("Video");
        let output_dir = root.path().join("Document");
        let working_dir = root.path().join("Working");
        let bin_dir = root.path().join("bin");
        fs::create_dir_all(&input_dir).expect("failed to create input dir");
        fs::create_dir_all(&output_dir).expect("failed to create output dir");
        fs::create_dir_all(&working_dir).expect("failed to create working dir");
        fs::create_dir_all(&bin_dir).expect("failed to create bin dir");

        let ffprobe_path = write_fake_tool(&bin_dir, "ffprobe", ffprobe_script());
        let ffmpeg_path = write_fake_tool(&bin_dir, "ffmpeg", ffmpeg_script());

        let old_path = std::env::var("PATH").ok();
        let old_api_key = std::env::var("MOONSHOT_API_KEY").ok();
        let old_ffprobe_bin = std::env::var("VIDEO2DOCUMENT_FFPROBE_BIN").ok();
        let old_ffmpeg_bin = std::env::var("VIDEO2DOCUMENT_FFMPEG_BIN").ok();
        let separator = if cfg!(windows) { ';' } else { ':' };
        let path_value = match &old_path {
            Some(existing) if !existing.is_empty() => {
                format!("{}{}{}", bin_dir.display(), separator, existing)
            }
            _ => bin_dir.display().to_string(),
        };
        unsafe {
            std::env::set_var("PATH", path_value);
            std::env::set_var("MOONSHOT_API_KEY", "test-api-key");
            std::env::set_var("VIDEO2DOCUMENT_FFPROBE_BIN", &ffprobe_path);
            std::env::set_var("VIDEO2DOCUMENT_FFMPEG_BIN", &ffmpeg_path);
        }

        let config_path = root.path().join("config.toml");
        write_config(
            &config_path,
            &input_dir,
            &output_dir,
            &working_dir,
            base_url,
        );

        Self {
            _guard: guard,
            _root: root,
            input_dir,
            output_dir,
            working_dir,
            config_path,
            old_path,
            old_api_key,
            old_ffprobe_bin,
            old_ffmpeg_bin,
        }
    }

    pub fn cli_args(&self) -> CliArgs {
        CliArgs {
            config: Some(self.config_path.clone()),
            input_dir: None,
            output_dir: None,
            working_dir: None,
            segment_seconds: None,
            retry_count: None,
            upload_mode: None,
            only_index: false,
            force: false,
        }
    }

    pub fn write_video(&self, relative: &str) {
        let path = self.input_dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create input parent dir");
        }
        fs::write(path, b"fake-video-bytes").expect("failed to write fake video");
    }

    pub fn write_text_file(&self, relative: &str, content: &str) {
        let path = self.input_dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create input parent dir");
        }
        fs::write(path, content).expect("failed to write text file");
    }

    pub fn read_output(&self, relative: &str) -> String {
        fs::read_to_string(self.output_dir.join(relative)).expect("failed to read output markdown")
    }

    pub fn rewrite_base_url(&self, base_url: &str) {
        write_config(
            &self.config_path,
            &self.input_dir,
            &self.output_dir,
            &self.working_dir,
            base_url,
        );
    }

    pub fn root_path(&self) -> &Path {
        self._root.path()
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        unsafe {
            match &self.old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
            match &self.old_api_key {
                Some(value) => std::env::set_var("MOONSHOT_API_KEY", value),
                None => std::env::remove_var("MOONSHOT_API_KEY"),
            }
            match &self.old_ffprobe_bin {
                Some(value) => std::env::set_var("VIDEO2DOCUMENT_FFPROBE_BIN", value),
                None => std::env::remove_var("VIDEO2DOCUMENT_FFPROBE_BIN"),
            }
            match &self.old_ffmpeg_bin {
                Some(value) => std::env::set_var("VIDEO2DOCUMENT_FFMPEG_BIN", value),
                None => std::env::remove_var("VIDEO2DOCUMENT_FFMPEG_BIN"),
            }
        }
    }
}

pub async fn mount_success_mocks(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/v1/files"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": "file_123" })),
        )
        .mount(server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_chat_response("语法总览")))
        .mount(server)
        .await;
}

#[allow(dead_code)]
pub async fn mount_failing_chat(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/v1/files"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": "file_123" })),
        )
        .mount(server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("upstream error"))
        .mount(server)
        .await;
}

pub fn success_chat_response(topic: &str) -> serde_json::Value {
    serde_json::json!({
        "choices": [
            {
                "message": {
                    "content": serde_json::to_string(&serde_json::json!({
                        "course": "英语语法精讲",
                        "relative_path": "CourseA/01_绪论.mp4",
                        "segment_id": "clip_001",
                        "time_range": "00:00:00-00:00:12",
                        "topic": topic,
                        "summary": "总结英语语法学习的整体框架与复习重点。",
                        "key_concepts": [
                            { "term": "句子成分", "note": "理解主谓宾和修饰语的角色。" },
                            { "term": "时态", "note": "关注动作发生时间及状态变化。" }
                        ],
                        "formulas": [
                            { "expression": "主语 + 谓语 + 宾语", "description": "基础句型结构。" }
                        ],
                        "teacher_emphasis": ["先掌握框架，再细看特殊情况。"],
                        "examples": [
                            { "title": "基础例句", "summary": "通过简单句说明语法框架如何落地。" }
                        ],
                        "confusions": ["待核对被动语态与时态的交叉用法。"],
                        "questions": ["为什么句子成分分析是语法学习的第一步？"],
                        "tags": ["语法", "框架", "复习"],
                        "low_confidence": ["部分例句的语义细节待核对。"]
                    }))
                    .expect("failed to serialize mock response")
                }
            }
        ]
    })
}

fn write_config(
    config_path: &Path,
    input_dir: &Path,
    output_dir: &Path,
    working_dir: &Path,
    base_url: &str,
) {
    let base_url = ensure_v1_base_url(base_url);
    let content = format!(
        "[paths]\ninput_dir = \"{}\"\noutput_dir = \"{}\"\nworking_dir = \"{}\"\n\n[video]\nsegment_seconds = 600\nmax_resolution_width = 2048\nmax_resolution_height = 1080\npreferred_upload_mode = \"file\"\n\n[kimi]\nmodel = \"test-model\"\nbase_url = \"{}\"\nmax_retries = 0\nretry_backoff_seconds = 0\n\n[runtime]\npreprocess_concurrency = 1\nrequest_concurrency = 1\nlog_filter = \"warn\"\n",
        normalize_path(input_dir),
        normalize_path(output_dir),
        normalize_path(working_dir),
        base_url,
    );
    fs::write(config_path, content).expect("failed to write config file");
}

fn ensure_v1_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_fake_tool(bin_dir: &Path, name: &str, script: String) -> PathBuf {
    let file_name = if cfg!(windows) {
        format!("{name}.cmd")
    } else {
        name.to_string()
    };
    let path = bin_dir.join(file_name);
    fs::write(&path, script).expect("failed to write fake tool");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("failed to stat fake tool")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("failed to chmod fake tool");
    }
    path
}

fn ffprobe_script() -> String {
    if cfg!(windows) {
        "@echo off\necho {\"streams\":[{\"width\":1280,\"height\":720}],\"format\":{\"duration\":\"12.0\"}}\n"
            .to_string()
    } else {
        "#!/usr/bin/env sh\necho '{\"streams\":[{\"width\":1280,\"height\":720}],\"format\":{\"duration\":\"12.0\"}}'\n"
            .to_string()
    }
}

fn ffmpeg_script() -> String {
    if cfg!(windows) {
        "@echo off\nsetlocal EnableDelayedExpansion\nset \"last=\"\nfor %%a in (%*) do set \"last=%%~a\"\nfor %%F in (\"!last!\") do if not exist \"%%~dpF\" mkdir \"%%~dpF\"\n> \"!last!\" echo fake-clip\n"
            .to_string()
    } else {
        "#!/usr/bin/env sh\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\nmkdir -p \"$(dirname \"$last\")\"\nprintf 'fake-clip' > \"$last\"\n"
            .to_string()
    }
}
