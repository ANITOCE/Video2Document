# Video2Document

[English](README.md) | [简体中文](README.zh-CN.md)

Video2Document 是一个命令行工具，批量的将本地存储的视频转写为相关文档(知识库)，调用 Kimi 多模态分析视频，为每个视频生成一份 Markdown 学习文档。

## 项目功能

- 递归扫描输入目录中的视频文件
- 自动跳过隐藏文件、临时文件和非视频文件
- 每个视频只生成一个与原视频同名的 Markdown 文档
- 所有文档都使用固定的章节顺序
- 将可续跑的状态和缓存写入独立的 Working 目录
- 在输出目录中生成 index.md，方便按目录逐级浏览

当前支持的视频后缀：

- .mp4
- .mov
- .mkv
- .avi
- .webm
- .m4v

生成的课程文档固定包含以下章节：

1. 本讲目标
2. 知识主线
3. 核心概念
4. 公式与结论
5. 例题与案例
6. 老师强调内容
7. 术语与概念简记
8. 问答与自测

## 运行要求

运行前请先确认：

- 已安装 Rust stable
- ffmpeg 已加入 PATH
- ffprobe 已加入 PATH
- 当前 shell 已设置有效的 MOONSHOT_API_KEY

PowerShell 示例：

```powershell
$env:MOONSHOT_API_KEY = "你的-kimi-api-key"
```

不要把 API key 写入任何受 Git 跟踪的文件。

## 如何运行

### 使用默认目录直接运行

如果仓库根目录下已经有 Video 目录，直接执行：

```powershell
cargo run --release --
```

默认目录如下：

- 输入目录：./Video
- 输出目录：./Document
- Working 目录：./Working

### 显式指定目录运行

```powershell
cargo run --release -- --input-dir "./Video" --output-dir "./Document" --working-dir "./Working"
```

### 做一个小范围 smoke test

如果你只想先试跑一个短视频或少量视频：

```powershell
cargo run --release -- --input-dir "./VideoSmoke" --output-dir "./DocumentSmoke" --working-dir "./WorkingSmoke" --segment-seconds 1800 --upload-mode base64
```

### 只重建索引

如果 Markdown 已经生成，只想重建索引：

```powershell
cargo run --release -- --output-dir "./Document" --working-dir "./Working" --only-index
```

## 常用命令行参数

| 参数 | 说明 |
| --- | --- |
| --input-dir | 覆盖输入视频目录 |
| --output-dir | 覆盖 Markdown 输出目录 |
| --working-dir | 覆盖缓存与状态目录 |
| --segment-seconds | 覆盖视频切片长度 |
| --retry | 覆盖可恢复失败的重试次数 |
| --upload-mode | 选择 auto、base64 或 file |
| --only-index | 不重新处理视频，只重建索引 |
| --force | 忽略已有缓存并重新处理 |

## 输出结构

输出目录示例：

```text
Document/
  index.md
  CourseA/
    index.md
    01_绪论.md

Working/
  CourseA/
    01_绪论/
      status.json
      metadata.json
      clips/
      kimi_segment_outputs/
      kimi_final_outputs/
```

## 断点续跑

如果运行中断，直接再次执行同一条命令即可。

- 当源视频和处理参数未变化时，已完成的视频会被复用
- 失败视频会在 Working 目录中保留状态和错误信息
- 如果你希望强制重新处理，可以使用 --force

## License

See [LICENSE](LICENSE).
