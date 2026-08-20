# 更新记录

## 0.1.0

### 新增

- 可选的 `auth.provider.trusted_ca_file`，用于使用私有证书颁发机构的 OIDC issuer。
- 内置 OIDC Provider，提供原生 Basic 登录、code/PKCE、持久签名与 refresh 轮换；内置／外部 issuer 共用单一 TokenSet／Bearer 业务模型。
- 独立 auth-provider 与共享数据库／配置 Rust crates、能力受限连接池及 owner 作用域 GraphQL 事务。
- Bearer 认证的原生图片／RSS 凭证、本地 macaroon 衰减、有限续期／重试与局域网 HTTP Crypto bootstrap。
- 持久化任务 runs/outbox、fence 执行、取消恢复与时区感知 Cron 调度。
- 原生配置插值、文件 secrets 和脱敏诊断。
- 带版本的图片衍生、有界媒体执行与来源可追溯的 backfill。
- 跨平台开发与发布工具，以及应用、迁移和平台验证。
- 基于 README 的双语 VitePress 文档站、本地搜索与 konobangu.apeiraco.com 的 GitHub Pages 发布。

### 调整

- 默认分支独占 CI 缓存写入，阶段间复用工具缓存，保存精简的 Cargo 第三方产物，并共享容器下载缓存。
- 升级依赖，采用 injection-js、SecurityDept RxSignal、Apollo 和 TanStack Table 9。
- 集中 HTTP 中间件、扩展 SDK 的唯一 WebUI environment（原生 Web 与测试宿主）与唯一资源所有权，使用 SDK callback 路由。
- 按应用作用域、认证和任务控制命名数据库权限；从单个数据库 URL 初始化 NOLOGIN 能力角色，由 Rust 迁移标识符生成授权，合并未发布迁移。
- 使用 RFC3339 时间戳与 Temporal，启动前加载缺失能力；浏览器基线为 Chrome／Edge 140+、Firefox 140+ 与 Safari／iOS 18.6+。
- 封面使用锁定的 JPXL Balanced 77 与静态 WebP80，通过自有 Rayon pool 上的单一有界执行器完成；保留原图，透明封面合成白底。
- 保留生成 bindings 的格式，Node 工具使用原生 ESM 路径元数据。
- 使用 Vite 邮件预览与导出、Playwright 管理 fixtures、unit/integration/e2e 测试目录，并共享 Rust 任务迁移标识符。
- Just 与 CI 共享按资源划分的验证阶段，隔离浏览器构建，通过显式命令检查／watch 同一 TypeScript 引用图。
- 验证仅由默认/release 分支推送及目标为这两个分支的 PR 触发，提供统一分支保护汇总检查。
- 单一已测来源 workflow 发布 metadata 选中的 bundle／运行镜像／fixture，绑定 run／attempt、元数据与不可变制品回执。
- 整理 CLI/Just 工具和生成输出，恢复 Zellij sessions，使用 kebab-case 可执行文件名，配置按服务分组并标明单位。

### 修复

- 修正发布闭环判定和自动角色初始化部署说明，将未完成交付保留为当前版本必交项。
- 独立站点 WebUI/API 使用 SDK TokenSet 边界、仅 Provider 使用的浏览器原生凭据、JWT 本地校验与本地退出／重连，按请求选择允许的 WebUI 来源。
- 分离 Cron 计划与执行字段/重试，删除无调用的通知机制。
- 使用官方 Vite 集成加载 GraphiQL worker，开发依赖保留在源码图。
- Vite 构建直接解析工作区源码，使独立浏览器 fixture 无需预先生成 recorder 声明。
- 响应头 preset 保留端点安全／缓存策略；图片格式协商保留授权、validator 与 Range 行为。
- 任务发布、取消、重试和删除受事务与租约 fence 保护。
- 干净 CI 检出先安装锁定 CLI 依赖再运行工具，并应用文件日志阈值。
- 发布元数据更新保留清单换行格式，发布说明支持 CRLF。
- 通过服务器 hash 核验处理 qBittorrent 重复添加冲突，并按 digest 固定集成测试镜像。
- 媒体任务先释放预算再通知完成，避免后续任务因资源释放竞态被错误限流。

### 移除

- 业务 API Basic 认证、WebUI Basic context／iframe 适配、cookie session runtime、浏览器 token 持久化、私有资源 Service Worker／PWA、Tera、Jotai/DI forks 与未使用的日期适配器。
- Codec 子进程、libjxl 构建流程和受版本控制的编译器声明产物。

### 升级说明

备份数据库与存储，停止写入者和 workers，并执行 owner-only 迁移；早期 Apalis 0.7 schema 中排队的任务不迁移，需先删除该 schema 或重建数据库；数据库回滚需要恢复备份。默认启动从单个数据库 URL 初始化受限连接池，也可使用独立运行或 owner 凭据。替换配置模板、平铺 auth/task/media 字段和旧 JXL 选项。自动封面保持 JXL + WebP；JPXL 当前输出普通单 pass 图片。

参见[认证](002-AUTHENTICATION-DECISION.md)、[升级](003-TASK-DELIVERY-AND-MIGRATION.md)和[媒体与配置](004-MEDIA-AND-CONFIGURATION.md)。

[English](../../CHANGELOG.md) | [中文](CHANGELOG.md)
