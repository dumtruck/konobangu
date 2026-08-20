# Konobangu

<img src="../../assets/icon.png" alt="Konobangu" width="160" />

自托管番剧订阅与录制服务：监控配置的番剧源，通过 BT 客户端下载匹配分集，并用 Web UI 浏览收藏。基于 Rust recorder、PostgreSQL 和 React WebUI，仍处于早期阶段。

## 安装与运行

应用 bundle 包含一个 `recorder-cli` 可执行文件和 `webui/` 目录。图片编码器与 TLS 编译进可执行文件，WebUI 从磁盘托管。运行需要 PostgreSQL 16+，认证可使用内置 OIDC provider 或外部 issuer。内置 Provider 使用浏览器原生 Basic 登录，业务 API 始终使用 Bearer。尚未发布带 tag 的公开版本，未关闭的交付要求见[发布验收](005-RELEASE-ACCEPTANCE.md)。目前从源码准备并验收 bundle：

```sh
mise install                       # Node, pnpm, Rust, just and other pinned tools
just setup                         # frozen installs, workspace checks and git hooks
just release prepare --target native # verifies and builds the complete application bundle
```

在 prepare 输出的平台目录中操作。将 `recorder.config.toml` 放在可执行文件旁，或通过 `--config-file` 指定，至少提供数据库 owner URL 和一个认证 provider。全部字段见[配置](004-MEDIA-AND-CONFIGURATION.md#配置来源)与[认证](002-AUTHENTICATION-DECISION.md#配置与角色)，包括通过文件提供密码和连接串等秘密配置的方式：

```toml
[database]
url = "postgres://konobangu:PASSWORD@localhost/konobangu"

[auth.provider]
type = "oidc" # External issuer; bundled setup is documented below.
issuer = "https://idp.example/"

[auth.provider.browser]
client_id = "konobangu-webui"

[auth.webui]
allowed_origins = [{ type = "strict", value = "https://ui.konobangu.example" }]
```

内置单管理员 Provider 使用 `type = "bundled"`，配置登记回调、凭据及持久 RSA 私钥，见[配置与密钥生成](002-AUTHENTICATION-DECISION.md#配置与角色)。不需要单独认证进程或服务。

运行可执行文件；启动时自动建立受限能力角色、执行迁移并打开连接池：

```sh
./recorder-cli --environment production --config-file recorder.config.toml
```

## 文档

[文档站点](https://konobangu.apeiraco.com/zh/)

| 主题 | 文档 |
| --- | --- |
| 开发、平台构建与验证 | [指南](001-DEVELOPMENT-VERIFICATION.md) |
| 认证与数据访问 | [契约](002-AUTHENTICATION-DECISION.md) |
| 任务投递与数据库升级 | [运维](003-TASK-DELIVERY-AND-MIGRATION.md) |
| 媒体与配置 | [参考](004-MEDIA-AND-CONFIGURATION.md) |
| 发布交付与未关闭验收 | [状态与契约](005-RELEASE-ACCEPTANCE.md) |
| 路线图与后续工作 | [路线图](roadmap/001-SHORT-TERM-ROADMAP.md) |
| Animeta 模型提案 | [提案](roadmap/002-ANIMETA-MODEL.md) |

运行主题指南描述当前行为，发布验收单列未关闭交付；`docs/{en,zh}/roadmap/` 维护未来决策与提案。发布历史见 [CHANGELOG](CHANGELOG.md)，Agent 与贡献规则见 [AGENTS.md](../../AGENTS.md)。

### 贡献与分享

欢迎在 [GitHub](https://github.com/apeiraco/konobangu) 提交 Issue 与 PR。提交前请按[开发指南](001-DEVELOPMENT-VERIFICATION.md)准备 Windows/macOS/Linux 工具、安装锁定依赖并运行 `just verify`。当前契约放在 `docs/`，未来提案放在 `docs/*/roadmap/`，交付历史放在 [CHANGELOG.md](CHANGELOG.md)；完整项目与文档规则见 [AGENTS.md](../../AGENTS.md)。分享本项目遵守许可证即可，无需额外许可。

## 仓库结构

- `apps/recorder`：Rust HTTP/GraphQL 服务、迁移、任务投递和有界进程内图片编码。
- `apps/webui`：React UI、SecurityDept 认证与 RxSignal、injection-js DI 和 Apollo GraphQL 数据。
- `apps/docs`：VitePress 文档站点，通过相对符号链接投影仓库文档真源。
- `apps/proxy`、`apps/email-playground`：开发工具。
- `packages/`：独立 auth-provider、共享数据库／配置 crates、下载器、元数据、邮件和测试辅助库。
- `scripts/dev-cli.mts`：跨平台 TypeScript 开发工具，由 `just` 调度。
- `justfiles/`：安装、开发、构建、质量、测试、发布和工具 recipes。
- `build/`、`deploy/`：Docker 构建输入和许可证通知。

## 许可证与致谢

采用 MIT 许可证，见 [LICENSE](../../LICENSE)。发行制品包含基于锁定依赖图生成的第三方许可证通知，见 [deploy/licenses](../../deploy/licenses)。Konobangu 内嵌同样采用 MIT 协议的 [JPXL](https://github.com/liminalism/JPXL) 编码 JPEG XL，并静态链接 `libwebp`/`sharpyuv` 编码 WebP。贡献通过 Git 历史记录，发布说明见 [CHANGELOG.md](CHANGELOG.md)。

[English](../../README.md) | [中文](README.md)
