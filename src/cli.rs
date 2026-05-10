use std::path::PathBuf;

use clap::Parser;

use crate::models::UploadMode;

#[derive(Debug, Clone, Parser)]
#[command(name = "video2document")]
#[command(about = "将视频批量转换为课程 Markdown 文档")]
pub struct CliArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub input_dir: Option<PathBuf>,

    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    #[arg(long)]
    pub working_dir: Option<PathBuf>,

    #[arg(long)]
    pub segment_seconds: Option<u64>,

    #[arg(long = "retry")]
    pub retry_count: Option<u32>,

    #[arg(long)]
    pub upload_mode: Option<UploadMode>,

    #[arg(long)]
    pub only_index: bool,

    #[arg(long)]
    pub force: bool,
}
