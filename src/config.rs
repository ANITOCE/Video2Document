use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cli::CliArgs;
use crate::models::UploadMode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    pub paths: PathSettings,
    pub video: VideoSettings,
    pub kimi: KimiSettings,
    pub runtime: RuntimeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathSettings {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub working_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoSettings {
    pub segment_seconds: u64,
    pub max_resolution_width: u32,
    pub max_resolution_height: u32,
    pub preferred_upload_mode: UploadMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KimiSettings {
    pub model: String,
    pub base_url: String,
    pub max_retries: u32,
    pub retry_backoff_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSettings {
    pub preprocess_concurrency: usize,
    pub request_concurrency: usize,
    pub log_filter: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub settings: AppSettings,
    pub only_index: bool,
    pub force: bool,
}

#[derive(Debug, Default, Deserialize)]
struct ConfigPatch {
    #[serde(default)]
    paths: PathSettingsPatch,
    #[serde(default)]
    video: VideoSettingsPatch,
    #[serde(default)]
    kimi: KimiSettingsPatch,
    #[serde(default)]
    runtime: RuntimeSettingsPatch,
}

#[derive(Debug, Default, Deserialize)]
struct PathSettingsPatch {
    input_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    working_dir: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct VideoSettingsPatch {
    segment_seconds: Option<u64>,
    max_resolution_width: Option<u32>,
    max_resolution_height: Option<u32>,
    preferred_upload_mode: Option<UploadMode>,
}

#[derive(Debug, Default, Deserialize)]
struct KimiSettingsPatch {
    model: Option<String>,
    base_url: Option<String>,
    max_retries: Option<u32>,
    retry_backoff_seconds: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct RuntimeSettingsPatch {
    preprocess_concurrency: Option<usize>,
    request_concurrency: Option<usize>,
    log_filter: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            paths: PathSettings {
                input_dir: PathBuf::from("./Video"),
                output_dir: PathBuf::from("./Document"),
                working_dir: PathBuf::from("./Working"),
            },
            video: VideoSettings {
                segment_seconds: 600,
                max_resolution_width: 2048,
                max_resolution_height: 1080,
                preferred_upload_mode: UploadMode::Auto,
            },
            kimi: KimiSettings {
                model: "kimi-k2.6".to_string(),
                base_url: "https://api.moonshot.cn/v1".to_string(),
                max_retries: 3,
                retry_backoff_seconds: 5,
            },
            runtime: RuntimeSettings {
                preprocess_concurrency: 2,
                request_concurrency: 2,
                log_filter: "info".to_string(),
            },
        }
    }
}

impl AppSettings {
    fn apply_patch(&mut self, patch: ConfigPatch) {
        if let Some(value) = patch.paths.input_dir {
            self.paths.input_dir = value;
        }
        if let Some(value) = patch.paths.output_dir {
            self.paths.output_dir = value;
        }
        if let Some(value) = patch.paths.working_dir {
            self.paths.working_dir = value;
        }

        if let Some(value) = patch.video.segment_seconds {
            self.video.segment_seconds = value;
        }
        if let Some(value) = patch.video.max_resolution_width {
            self.video.max_resolution_width = value;
        }
        if let Some(value) = patch.video.max_resolution_height {
            self.video.max_resolution_height = value;
        }
        if let Some(value) = patch.video.preferred_upload_mode {
            self.video.preferred_upload_mode = value;
        }

        if let Some(value) = patch.kimi.model {
            self.kimi.model = value;
        }
        if let Some(value) = patch.kimi.base_url {
            self.kimi.base_url = value;
        }
        if let Some(value) = patch.kimi.max_retries {
            self.kimi.max_retries = value;
        }
        if let Some(value) = patch.kimi.retry_backoff_seconds {
            self.kimi.retry_backoff_seconds = value;
        }

        if let Some(value) = patch.runtime.preprocess_concurrency {
            self.runtime.preprocess_concurrency = value;
        }
        if let Some(value) = patch.runtime.request_concurrency {
            self.runtime.request_concurrency = value;
        }
        if let Some(value) = patch.runtime.log_filter {
            self.runtime.log_filter = value;
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.video.segment_seconds == 0 {
            bail!("segment_seconds must be greater than 0");
        }
        if self.runtime.preprocess_concurrency == 0 || self.runtime.request_concurrency == 0 {
            bail!("runtime concurrency must be greater than 0");
        }
        if self.kimi.model.trim().is_empty() {
            bail!("kimi.model must not be empty");
        }
        Ok(())
    }

    pub fn processing_fingerprint(&self) -> Result<String> {
        #[derive(Serialize)]
        struct Fingerprint<'a> {
            video: &'a VideoSettings,
            kimi_model: &'a str,
        }

        let payload = Fingerprint {
            video: &self.video,
            kimi_model: &self.kimi.model,
        };
        let json = serde_json::to_vec(&payload)?;
        let mut hasher = Sha256::new();
        hasher.update(json);
        Ok(hex::encode(hasher.finalize()))
    }
}

pub fn load_settings(args: &CliArgs) -> Result<ResolvedConfig> {
    let base_path = args
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from("config.toml"));
    let local_path = base_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("config.local.toml");

    load_settings_from_paths(
        Some(&base_path),
        Some(&local_path),
        args,
        args.config.is_some(),
    )
}

pub fn load_settings_from_paths(
    base_path: Option<&Path>,
    local_path: Option<&Path>,
    args: &CliArgs,
    base_is_explicit: bool,
) -> Result<ResolvedConfig> {
    let mut settings = AppSettings::default();

    if let Some(path) = base_path {
        if path.exists() {
            settings.apply_patch(read_patch(path)?);
        } else if base_is_explicit {
            bail!("config file not found: {}", path.display());
        }
    }

    if let Some(path) = local_path
        && path.exists()
    {
        settings.apply_patch(read_patch(path)?);
    }

    apply_cli_overrides(&mut settings, args);
    settings.validate()?;

    Ok(ResolvedConfig {
        settings,
        only_index: args.only_index,
        force: args.force,
    })
}

fn apply_cli_overrides(settings: &mut AppSettings, args: &CliArgs) {
    if let Some(value) = &args.input_dir {
        settings.paths.input_dir = value.clone();
    }
    if let Some(value) = &args.output_dir {
        settings.paths.output_dir = value.clone();
    }
    if let Some(value) = &args.working_dir {
        settings.paths.working_dir = value.clone();
    }
    if let Some(value) = args.segment_seconds {
        settings.video.segment_seconds = value;
    }
    if let Some(value) = args.retry_count {
        settings.kimi.max_retries = value;
    }
    if let Some(value) = args.upload_mode {
        settings.video.preferred_upload_mode = value;
    }
}

fn read_patch(path: &Path) -> Result<ConfigPatch> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let patch = toml::from_str::<ConfigPatch>(&raw)
        .with_context(|| format!("failed to parse TOML config {}", path.display()))?;
    Ok(patch)
}
