# 005 — 0.1.0 发布交付与验收

## 当前状态

**内置 OIDC 与统一认证模型已通过当前宿主的行为复验；运行镜像验收（RC-003）与公开发布（RC-007）仍未闭环。** 本文管理当前版本交付与未关闭验收，不将其转移到未来 roadmap；公开发布必须绑定最终提交及同一成功 CI run 和制品。

| ID | 状态 | 必须完成的交付 |
| --- | --- | --- |
| RC-001 | 实现验收通过；公开执行归 RC-007 | 已测来源／精确 attempt、metadata 选择、reusable 发布、冲突拒绝和同候选恢复通过独立审查与行为测试 |
| RC-002 | 本地制品验收通过；CI 来源归 RC-007 | 四原生目标完成构建和执行；共享工具对真实压缩包完成组装／解压、哈希、配置、WebUI 与完整 notices 检查 |
| RC-003 | 认证改造后须复验 | 运行镜像须覆盖内置／外部 OIDC、空卷初始化、真实任务／媒体、持久化、非 root／只读运行及优雅退出；公开分发属于 RC-007 |
| RC-004 | 当前宿主复验通过 | 完整 OIDC 应用通过真实 GraphiQL editor／JSON／GraphQL 操作及 worker 成功回复配对、字体加载、组件／根 HMR、草稿保留与 Vite 停启；本地 SDL 与 document 变化无需 token 即可重新生成类型 |
| RC-005 | 已关闭 | 用户已取消并行抽象恢复要求；保持现有自有 Rayon pool |
| RC-006 | 当前宿主复验通过 | 显式管理员通过真实 API／页面编辑计划与重试关联运行；隔离 PostgreSQL 测试通过修订后的迁移及四项能力边界 |
| RC-007 | 管理者负责 | 2026-10-07 已核验现有外部设置；同一候选的实际发布/失败恢复证据仍由管理者负责 |
| RC-008 | 当前宿主复验通过 | 真实 Chromium、Firefox、WebKit 通过内置／外部 OIDC、仅 Provider Basic 登录、纯 Bearer API、LAN HTTP WebUI/API + HTTPS issuer、原生图片／RSS 及有限重试，保留原生随机源和 Chromium 140 基线；WebKit 使用开发验证中说明的限定登录驱动 |

GraphiQL lazy route 使用 `graphiql/setup-workers/vite`；worker adapter 保留在 Vite 源码图，编辑器运行时入口与 CommonJS 叶依赖显式优化。开发时提前扫描 bootstrap 和 lazy route 源码，GraphQL／React 解析为唯一包实例，保证 schema 对象与语言服务兼容；这些开发优化不提前加载生产编辑器。验收执行真实 editor／JSON／GraphQL 操作，以及完整应用的 Vite 配置、入口和 HMR 路径，setup console label 本身不足。

Cron 是计划，执行结果与重试归属 task_runs。计划保留启停、表达式、时区、下一次/上次调度 occurrence 及调度诊断；API 不再提供 priority/timeout/maxAttempts 或执行状态输入。失败执行从关联任务进入既有 retry 命令，任务执行器保留固定三次尝试。轮询保证调度正确性，已删除无调用的旧 NOTIFY 函数；last_error 中的 occurrence 跳过属于调度诊断，不是任务失败。

发行使用同一 WebUI，在认证服务创建前取得运行时 API 地址与公开认证配置并绑定到 DI；无法读取或非法配置明确失败。公开配置始终投影 OIDC，内置／外部选择只属于服务器私有配置；预认证 fixture 不能证明真实 code/PKCE 或分域可用。

本地 release 工具创建不可公开发布的预览；唯一 release orchestrator 校验来源和 CI 输入、预留 draft、核验下载产物、完成发布及稳定别名提升。预览与维护者重试见 [开发验证](001-DEVELOPMENT-VERIFICATION.md)。

## 验收边界

原生 Windows/MSVC、macOS、Linux GNU/musl 的构建和 codec 执行证据可以关闭平台实现要求；平台无关单测在一个适当 runner 验证，不要求因任意改动重复所有平台测试。编译、Clippy、生产 WebUI 构建与隔离 HMR 分别证明各自范围。开发服务、最终打包和实际发布需运行对应真实入口。

发布完成须同时满足上表的实现要求、下面的发布选择/事务契约，以及实际候选的执行证据。测试和脚本覆盖全关闭、各单项、组合选择、错误来源、tag/版本冲突、上传失败和同候选重试；真实发布验收不通过生产 tag 故意制造冲突。公开版本不能被不同源码或选择覆盖。不得用缩小声明、fixture 替换真实入口或移入 roadmap 关闭漏项。

候选实现验收与公开发行分别判定：前者要求必交实现和真实入口验证齐备，后者由维护者执行发行时关闭 RC-007 的外部状态与结果证据。没有公开 Release 本身不证明代码错误，也不要求先发版才能修改版本号。本地压缩包与运行镜像回执保留 local／dirty 来源；orchestrator 仅接受最终提交源码的精确成功验证运行所产出的可发布制品。

Awa、模型工具链、Rust stable、真实渐进 JXL 的后续条件见 [roadmap](roadmap/001-SHORT-TERM-ROADMAP.md)，不扩入本次已约定的交付。JPXL 普通 JXL 默认不因渐进工作未完成而关闭。

## 独立部署 WebUI 与 API

WebUI 与 API 可位于无关 HTTPS 站点或静态托管，WebUI 保持 origin 根，API base URL 可带反代前缀。两种 issuer 来源均支持通过 Crypto barrier 的显式 LAN HTTP WebUI/API，issuer 保持固定 HTTPS。同一 executable/WebUI 共用 TokenSet／Bearer 模型。Origin/Referer 选中允许的 WebUI callback，issuer 再精确登记；Host/Forwarded/XFF 不决定 issuer/callback。

内置 Provider 位于 Recorder 可执行文件，使用自有协议 schema 和受限连接池。验收覆盖 discovery/JWKS、仅 `/auth-provider/login` 的原生 Basic、client/callback/scope 精确绑定、S256、nonce/ID token、过期／重放／并发授权码、refresh 轮换／family 重用及重启持久化。必须使用持久 RSA key。业务 API 全部拒绝 Basic，删除 `/api/login`；外部 issuer 直接连接，保留 SDK 真实到期／refresh／签名公钥轮换验证。

WebUI 复用 SecurityDept frontend-oidc SDK 与 callback，API 复用资源验证器，验证签名、issuer、audience、时间、scope、非对称算法、at+jwt 用途及非空 subject。ID/refresh token 不授权 API。外部 discovery/JWKS 由 SDK 缓存，内置公钥直接供应启动前 verifier，不引入每请求 IdP／应用 session 查询。保留 subscriber 映射、RLS、显式管理员和按身份 Apollo cache；历史共享 subscriber 不授予管理员。

退出清理本地 SDK material/principal，取消私有请求与刷新工作、停止旧 Apollo client、递增 epoch 拒绝晚到结果，不撤销已签 JWT／资源凭证，也不保证 IdP 全局退出。内置登录的浏览器缓存可能保留。公开客户端始终 omit credentials，无 API cookie／Basic fallback／自定义 CSRF header／浏览器凭据 store；只有 SDK 短期 redirect state 使用 sessionStorage。协议 grant state 属于 Provider，不是 resource server session。

environment 唯一拥有 SDK／应用资源，React 借用。既有 runner 验证 callback 清理与缺失／错误状态、shutdown/logout 竞态、principal 取消、安全重放与 HMR。开发 codegen/watch 默认消费 Recorder 导出的本地 SDL，无需 token；浏览器 fixture 显式选择 HTTP introspection，通过真实内置 code+S256 取得 Bearer，不绕过认证。受保护命令与 mutation 不自动重放。

原生图片／RSS URL 仅含衰减资源凭证，不含 access token 或父签名。覆盖父子过期、owner/path 隔离、304 前鉴权及不替换已加载图片的有界续期。失败图片先清旧 src，再且只发一次替换请求，包括 URL 相同情况。RSS 保留原生导航、owner RLS、本地续期及 XML self-link 不携带 query 凭据。分域测试禁用第三方 cookie，并覆盖 LAN HTTP、Firefox/WebKit 及 Chromium 基线。

## 发布自动化交付

采用 Android-Credential-Provider-Fixer 的已测源码 dispatch 模型，与 SecurityDept 的方向一致；Outposts 的镜像构建/CD 模型无法覆盖本项目的跨平台发行 bundle。以下内容作为一次完整交付，不增加按功能域拆分的验证组。

### Metadata 选择与发布历史

```toml
[release.artifacts]
bundles = true
runtime_image = true
testing_torrents_image = false
```

三个独立布尔值位于 `konobangu-metadata.toml`，字段缺失、类型错误或未知字段均拒绝。它们选择发布内容，不裁剪测试覆盖。全 false 表示仅验证，不 dispatch、不审批、不创建 tag 或 GitHub Release。仅镜像发布仍创建版本 tag 和附带 manifest/digest 的 GitHub Release，不要求 bundle 压缩包。不在仓库 variables 或 dispatch inputs 复制这些开关，也不提供覆盖已提交选择的命令参数。

`just release plan` 输出选择、source SHA、metadata SHA-256（统一检出换行后计算）与候选镜像标签，`--format github-output` 将同一计划提供给 job 条件。本地预览标记 dirty checkout；自动发布须从精确、干净、已测源码解析计划，重试也读取该 SHA 的 metadata，不读取分支新尖端。默认选中 bundle 与运行镜像，testing-torrents 显式 opt-in。release orchestrator 实现以下来源、制品及发布检查。

Git 历史证明的是发布意图，不能证明上传成功。release manifest 保存 metadata 快照/哈希、source SHA，以及实际成功的压缩包哈希、镜像 digest 和 workflow run ID。Bundle 回执已包含选择/哈希，本地发布拒绝 metadata 变化。可复用 fixture publisher 校验 release caller、真实候选计划和已提交的 opt-in 开关，导出 digest 与源码/配置哈希。候选镜像标签不含 `latest`，全部选中产物成功且稳定版完成后才统一提升该别名。同一版本公开后，重试不能改变选择或源码；调整发布范围应发布新版本，不重写历史。

draft 公开前，除已认证的候选检查外，还必须用匿名、只读 GHCR client 核验全部选中镜像的不可变引用、内容 digest 与可运行平台配置。包仍为 private 时阻止 finalization，保留可恢复的 draft；首次公开由维护者设置 package visibility，workflow 不虚构 visibility API，也不要求管理权限 PAT。公开版本重试仍做匿名核验，成功提升的稳定别名须指向相同镜像 digest。

### 发布内容

候选版本为 `0.1.0`，与 `konobangu-metadata.toml` 及双语 changelog 一致，对应 tag 为 `v0.1.0`。带版本的 changelog 不表示已完成发布。维护一个应用发行版本，不为每个 workspace 成员单独做 registry 发布。

| 交付项 | 发布决策 |
| --- | --- |
| 应用 bundle | 四个带 target 名称的压缩包：`x86_64-unknown-linux-gnu`、`x86_64-unknown-linux-musl`、`x86_64-pc-windows-msvc`、`aarch64-apple-darwin`。各包含 `recorder-cli`/`.exe`、`webui/`、LICENSE、CHANGELOG、notices 与 source/hash 回执。自动打包补充无 secrets 配置示例，不打包用户数据或凭据。 |
| 构建 profile | 发布默认 JXL + WebP，使用现有自有 Rayon pool；WebP-only 是构建/验证选项，不单独形成发行产品。 |
| 主运行镜像 | 发布 public `ghcr.io/apeiraco/konobangu`，首批为 `linux/amd64`，复用已验收 musl executable 与同版 WebUI。仅打包一次，使用包含 CA 证书的最小非 root runtime，不含 Cargo、Node 或 codec 构建工具。配置、secrets 和可写应用数据分别挂载；PostgreSQL 与 BT 客户端仍为外部服务；默认可内嵌 OIDC Provider，仅显式选择外部 issuer 时另需外部 IdP。 |
| 镜像标签与元数据 | 使用完整版本和 `sha-<source SHA>` 标签，稳定版完成后才移动 `latest`。release manifest 记录镜像 digest。附带 OCI source/revision/version/license labels，关联源仓库，首次发布显式启用 public 可见性。Linux arm64 在具备原生制品验证后再增加。 |
| 工作区库 | Node 包保持 private，Rust crates 不发布。`email`、`testing`、recorder bindings、`util`、`util-derive`、`fetch`、`downloader` 和 Rust `animeta`、`konobangu-auth-provider`、`konobangu-database`、`konobangu-configuration` 是内部构建依赖，交付应用，不拆为 npm/crates.io 发布。Python `konobangu-animeta` 与模型 demo 属于开发/研究工具，本期不发布 PyPI 包或模型。 |
| 开发二进制 | 不交付 `mikan-doppel`（mock 源）、`auth-test-server`、`testcontainers-prune`、examples 或 `animeta-agent-cli`（当前为空实现）。Email playground 预览/导出也是开发产物。 |
| 测试镜像 | `ghcr.io/apeiraco/konobangu-testing-torrents` 由 `release.artifacts.testing_torrents_image` 选择，release orchestrator 仅在选中时调用其 reusable publisher。默认 false，仍与主运行镜像职责不同。 |
| 构建镜像与文档 | musl builder 仅供 CI 本地使用，不作为用户镜像发布。文档由 `master` 的 Pages 流程发布，不将文档站或编译缓存/声明产物放进应用 bundle。 |

主运行镜像复用已验证 musl executable 和生产 WebUI，使用非 root distroless runtime。服务验收覆盖真实数据库初始化、认证、持久数据、只读 rootfs、重启与优雅退出。

1. `release` 推送的 `Verification report` 成功后，从已提交 metadata 解析选择，仅在至少一个产物被选中时携带精确 `source_sha`、`source_ref`、`tests_run_id` 和 `tests_run_attempt` dispatch `release.yaml`，仅 dispatch job 获得 `actions: write`。允许按相同输入手动重试；PR、`dev`、默认分支和无关联 tag 推送不自动发布。
2. 通过 GitHub API 验证源仓库、verification workflow 身份、已完成且成功的运行、push 事件、release 分支和一致 commit。要求元数据同步及精确版本 CHANGELOG。检出已测试 SHA，不读取可变分支尖端；依赖 dispatch 前，workflow 文件必须先进入默认分支。
3. 在现有平台 job 上传已验证 recorder executable、notices 和目标回执，在现有 TypeScript job 上传生产 WebUI。上传 executable 和 notices，不能仅保留 smoke 回执。各制品记录 source SHA、版本、target 与哈希。发布消费该已验证运行的制品，不重复构建或门禁；重构本地与 CI 打包以共享源码/制品校验，不提供手动 `--verified` 开关。
4. 仅在 `bundles` 选中时，为 Linux GNU/musl x64、Windows MSVC x64、macOS arm64 生成带 target 名称的压缩包，包含 WebUI、LICENSE、CHANGELOG 和第三方 notices。验证候选来源及文件哈希，再生成 release manifest 和 SHA256SUMS。
5. 稳定版通过 `release` ref 的 `stable-release` 审批后发布，部署环境仅允许该分支，不允许 tag 来源。预发布状态按 SemVer 元数据判断。仅 tag/发布 job 获得 `contents: write`，仅选中的镜像发布 job 获得 `packages: write`，使用 `GITHUB_TOKEN`，不需要 npm/crates/PyPI 发布或签名 secrets，仓库默认权限仍为只读。对已验收产物打包运行镜像并 smoke，发布版本/source 标签时拒绝覆盖不同候选，manifest 记录 digest，稳定发布成功后再移动 `latest`。[GitHub registry 鉴权](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry)
6. 缺失版本 tag 时在已测 SHA 创建，已存在且指向其它 commit 时拒绝。先创建 draft、下载校验上传结果，再公开 release。重试接受完全相同的已发布版本，拒绝不同源码或制品，不能覆盖公开 release。manifest 记录测试 run ID 与 source SHA。
7. 验收一次真实 release 分支推送到验证、审批、可下载 bundle 和 public 可拉取运行镜像的完整路径，以及失败/重试、tag 冲突。镜像验证启动、WebUI/API 路由、secret 文件配置与持久数据挂载，仅 codec smoke 不足。外部设置与发行结果由管理者核验；分支保护配置不等于发布成功。覆盖仅 bundle、仅运行镜像、仅 fixture 镜像、组合及全关闭的选择，不要求未选中的产物。fixture publisher 作为 reusable workflow 接入同一个 release orchestrator。

[English](../en/005-RELEASE-ACCEPTANCE.md) | [中文](005-RELEASE-ACCEPTANCE.md)
