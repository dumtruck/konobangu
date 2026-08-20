# 001 — 开发、构建与验证

## 工具与入口

在仓库根目录使用 mise 管理 Node、pnpm、Rust、just、CMake 和其它工具；Rust 由 `rust-toolchain.toml` 固定。`just` 加载 `.env`，协调 pnpm、Cargo、uv 与唯一自定义工具 `scripts/dev-cli.mts`。命令参考由 recipe 注释维护，使用 `mise exec -- just --list` 和 `mise exec -- just dev-cli --help` 查看。

先安装原生构建工具与 mise 工具，再安装冻结的 Node workspace 依赖，之后才能调用 dev-cli。`just setup` 完成受管工具、依赖和 hooks 安装；CI 共用 `.github/actions/setup` 完成对应前置准备。门禁不修改或升级 workspace 依赖；浏览器 fixture 使用固定且受 IdP 支持的 LTS Node 与浏览器版本。Chromium 覆盖完整协议，Firefox/WebKit 验证跨站认证与原生图片。所有平台与 CI 使用同一条浏览器 fixture 路径，不需要特权，也不修改操作系统的信任库或 hosts：fixture 使用项目的 dev 依赖（不依赖系统命令）在被忽略的 temp 目录生成短期 CA 与一张覆盖测试域的叶子证书，签发后即丢弃 CA 私钥；Chromium 通过启动参数只信任 fixture 叶子证书的公钥并映射测试域名；Firefox 与 WebKit 使用 Playwright 自带构建，经由 fixture 自有的回环代理（Playwright 的 proxy 选项）访问测试域名，并对该 fixture 证书跳过证书错误；fixture 自己的 request client 在进程内信任该 CA 并解析这些域名。测试 IdP 使用同一张证书在 `localhost` 上提供 HTTPS，因为 WebKit 会拦截 HTTPS 页面访问 HTTP issuer；API 服务端通过 `auth.provider.trusted_ca_file` 校验它。

WebKit 需要与锁定 Playwright bundle 匹配的运行库。浏览器 runner 默认一个 worker：Docker bridge 变化可能中断其它正在运行的浏览器，共享 worker 也能复用真实 server/WebUI 构建；可用 runner 参数显式调整资源上限。

| 平台 | 原生构建前置条件 |
| --- | --- |
| Windows 11 / Server 2022+ x64 | Visual Studio C++ Build Tools、Windows SDK、CMake、NASM、PowerShell 7.5+；在 MSVC 开发环境运行 |
| macOS arm64/x64 | Xcode Command Line Tools、CMake、NASM |
| Linux x64 | C compiler、CMake、NASM、pkg-config、OpenSSL 开发包 |
| Docker | Linux 主机或 Windows/macOS Docker Desktop；容器目标使用固定 linux/amd64 镜像 |

Node 依赖使用 pnpm；Rust 使用 Cargo；Python 使用 uv workspace。开发的外部依赖与普通业务、identity、scheduler 数据库连接按 [认证](002-AUTHENTICATION-DECISION.md) 配置。测试自行持有隔离资源，不复用开发数据库。

## 浏览器支持范围

WebUI 支持 Chrome／Edge 140+、Firefox 140+、Safari 18.6+，包括 iOS／iPadOS 18.6+。用户应使用设备可升级的最新浏览器或系统版本；冻结在应用中的内嵌浏览器和更旧的引擎不在支持契约内。Vite 的 JavaScript 与 CSS 构建目标使用该基线；通过 environment 的能力边界检查兼容性，不增加按浏览器版本封禁的逻辑。

只保留该基线仍需要的 bootstrap 能力适配：Temporal、explicit resource management，以及显式配置的 LAN HTTP 上的 Crypto 计算能力；必须保留原生随机源。升级浏览器不会使任意 HTTP origin 获得安全上下文专属 API；图片使用原生资源 token，不依赖 Service Worker。原生支持的能力不加载对应补丁 chunk。既有浏览器 runner 同时覆盖当前引擎与 Chromium 140；`just setup-browser` 安装固定 fixture，基线 Playwright 版本只在 Just 中定义。

## 执行模型

| 层 | 职责 |
| --- | --- |
| 原生入口 | `lint/check/test/build-*` 调用对应生态工具；runner 参数负责筛选具体测试 |
| 门禁 | `verify` 的资源阶段组合原生入口，检查清单仅在 `justfiles/test.just` 定义 |
| 平台制品 | `platform-check` 管理目标构建、依赖检查和实际制品 smoke |
| 发布 | `release prepare` 运行完整门禁和平台验收，再创建本地预览；release workflow 发布已验证的 CI 输入 |

`verify static` 检查元数据、工具行为、文档链接、格式和 diff；`verify rust` 使用 Cargo/Docker 验证编译、完整 workspace 与浏览器集成；`verify ts` 验证声明、bundler 产物、前端单测、lint 与输出边界。`verify` 按 static → rust → ts 执行，让 Rust bindings 和 GraphQL 生成器先于 TypeScript 消费者。CI 在独立检出中引用同一阶段入口；Rust 阶段拒绝生成 bindings 漂移。浏览器 fixture 在隔离目录构建同一 WebUI，以运行时配置验证内置／外部 OIDC、同源与独立托管 WebUI/API。内置 Basic 原生 challenge 仅在 HTTPS issuer 的 login 发生；GraphQL codegen 通过真实 code + S256 兑换取得 Bearer。

Chromium 与 Firefox 原生响应内置登录 challenge。WebKit 驱动器仅应答精确的顶层 Provider login：核验真实 401 与 realm，向同一 URL 发送 Basic，再将真实 302 交给浏览器；SDK code/PKCE、token 与 API 请求保持原样。这避免 WebKit 自动化凭据附加到 `credentials: "omit"` 的其它 fetch；Safari 原生凭据缓存行为仍需通过 Safari 验证。

阶段按资源划分且互不重叠。新增测试进入既有 Cargo、Vitest、Node 或 Playwright runner，不新增 recipe、CI job 或门禁步骤。只有新增工具链、feature 组合或外部依赖才增加步骤；出现覆盖重叠时合并。不要编写镜像 recipe 字面内容、job 数量或步骤顺序的测试；应验证参数传递、退出码、资源清理和制品行为。

`test rust` 默认执行 workspace 的库、binary、集成与 playground 示例，再执行 Cargo 在 all-targets 中排除的 doctest。显式 Cargo 参数替换默认选择，适合定位包、feature、test target 或名称过滤器；Vitest/Node 使用各自原生过滤参数。`test browser` 使用生产内置 Provider、真实外部 oidc-provider 和 Chromium 验证认证、polyfill 与生命周期。任意 runner 失败即停止并保留原始失败码；默认 Rust 执行顺序由 Just 管理。缺少 Docker 或构建前置条件应直接失败。

macOS 上 Recorder 的 debug 集成测试可执行文件使用 DWARF 展开，避免链接器 compact unwind 的 16 MiB 偏移上限；发行可执行文件与其它目标保留正常链接设置。

开发 Recorder 在监听前将当前 GraphQL SDL 原子导出到 `apps/recorder/.cache/graphql/schema.graphql`。`just dev-codegen` 与 `just dev-codegen watch` 默认使用该本地文件，watch 跟踪 schema／document，不需要 access token，也没有 token 过期问题；先启动开发 Recorder。`KONOBANGU_GRAPHQL_SCHEMA` 可显式选择其它 SDL 文件或 HTTP 端点，只有 HTTP 使用 `KONOBANGU_CODEGEN_AUTHORIZATION`。生产默认不导出，`--graphql-schema-output` 是显式 CLI 输出，不是公开 schema 路由；浏览器 fixture 继续使用真实 Bearer 保护的 HTTP introspection 验证认证边界。

qBittorrent 生命周期 fixture 在 `packages/downloader/tests/integration/qbit-lifecycle.test.rs` 固定多平台镜像 tag/digest。升级时显式更新该引用，并在 Docker Desktop 与原生 Linux 运行 downloader suite，覆盖认证、重复添加、sync、暂停/恢复、实际传输、保留/删除文件、超时与停机。WebAPI 2.14 可能以 HTTP 409 表示重复添加；适配器查询服务器，仅在该次提交的全部 hash 都存在时接受冲突。Mock 测试覆盖文件/magnet 冲突、过期本地缓存、批量部分缺失和其它 API 错误。[上游 API 变更](https://github.com/qbittorrent/qBittorrent/blob/master/WebAPI_Changelog.md#2140)

`lint rust` 检查 workspace 与裁剪后的 recorder，all-targets 且 -D warnings；codec 是唯一的编译期选择，这两个配置即覆盖全部有效组合。媒体编码使用唯一的 Rayon pool，构建时不再选择并行 backend。`test media` 构建 WebP-only 裁剪配置并验证其库、测试二进制与真实编解码，覆盖所有 `not(feature = "jxl")` 分支；`test rust` 覆盖发布所用的 JXL + WebP 构建。两者都是同一阶段的 Cargo 工作，因为 media 源码没有任何按操作系统条件编译的分支。各目标任务只验证平台之间真正不同的部分——原生工具链与链接、真实编解码执行、制品形态——不重复这部分编译覆盖。Windows/macOS/Linux 与 Docker 都必须有实际构建和运行证据。

允许的 lint 例外限定于具体 OIDC handler 与共享 fixture；其它有效 rustc/Clippy 和 Cargo manifest 告警失败。查看原始 proc-macro/linker 日志；-D warnings 不覆盖全部诊断来源。保留测试 IdP 的内存 adapter/TTL 提示、Docker 与许可证工具的诊断，不过滤日志伪造零告警。前端 bundle 阈值与初始/polyfill/editor 加载由配置及浏览器断言共同检查。

### GitHub 流水线

默认分支为 `master`。验证运行于目标为 `master` 或 `release` 的 PR，以及这些分支的推送。仅推送 `dev` 不触发验证；创建或更新它到上述目标的 PR 仍会触发。PR 分支过滤器匹配目标分支，见 [GitHub 事件说明](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request)。

只有 `master` 推送的验证任务写入共享工具与 Cargo 缓存；PR、release 运行和文档任务均只读恢复。`static` 根据 `mise.ci.toml` 安装 Linux CI 所需工具的合集，保存 mise/pnpm 缓存后，其它阶段才启动。受版本管理的 CI profile 参与 mise 缓存键，工具版本仍由根配置定义。消费者仅启用 `just`、Node、pnpm 和自己声明的额外工具；本地开发工具选择保持不变。其它原生操作系统各由一个平台任务维护 mise/pnpm 缓存。Rust 阶段独立维护浏览器缓存和 debug 依赖缓存，各平台任务维护对应目标的 release 依赖缓存。不同目标与 profile 保持隔离；冷缓存构建必须仍然可用。

CI 关闭 Cargo 增量编译和 dev/test 的 debug 信息。Rust 缓存保留第三方依赖，排除 workspace 产物与 Cargo 安装的工具，仅在默认分支任务成功后保存。Docker 构建与宿主工具共用可移植的 Cargo registry/git 源码，遵循 `CARGO_HOME`，并传递 CI profile 设置。testing-torrents Docker layers 使用独立的 `testing-torrents` scope 与 `mode=min`。不为缓存预热新增重复编译任务。GitHub 仓库配额涵盖当前与历史缓存；扩大缓存范围前，应根据运行后的压缩大小与命中率评估。[GitHub 缓存限制](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#usage-limits-and-eviction-policy)

`Verification report` 是唯一必需状态检查，汇总 static、TypeScript、Rust 和平台任务。失败、取消或跳过任一阶段均不能通过。新的 PR/主分支推送取消过时验证；release 推送保留各轮运行，使发布能够绑定精确候选。

文档在 `master` 上的输入变化时构建部署，也支持手动构建；部署由 `github-pages` 限定为 `master`。应用发布使用 Release workflow 和已提交的 metadata 选择。testing-torrents publisher 仅由该 orchestrator 携带已核验候选调用并导出 digest；其独立 Docker 构建使用固定工具版本及与开发一致的冻结 lockfile/build 策略。本地预览与维护者重试入口见下文和[发布交付与验收](005-RELEASE-ACCEPTANCE.md)。

仓库级保护、审批环境及 Pages 设置由管理者维护。workflow 必须约束来源 ref 和最小权限，并验证直推候选；版本与发布选择由项目 metadata 管理。实施者只维护本仓实现，不操作外部管理设置。

## 职责与目录

根 `justfile` 只维护全局设置、argv 适配和 imports；`justfiles/` 按 setup/dev/build/quality/test/release/tools 拆分。Just 直接表达原生命令与固定顺序，自带 if 选择固定 scope；复杂操作进入受 TypeScript/Biome 覆盖的 CLI。纯转发统一使用 `recipe_args`，Unix 使用原生 positional argv，Windows 使用 PowerShell 7.5+ CommandWithArgs 和 argv 数组，显式保留 LASTEXITCODE。cross-spawn 处理 Windows 命令 shim，不将参数重建为 shell 字符串。

CLI 管理有完整输入/输出及失败语义的文件、网络、元数据和制品操作，包括清理、TCP 就绪、参数适配、Docker 挂载和发行事务。固定门禁与矩阵仍由 Just 调度。不要添加通用 quality 包装、中间半成品命令或手动 verified 开关。pnpm scripts 只保留必要生态生命周期；移除非冗余命令前须有可用替代入口。

`just dev-all` 与 `just dev-proxy` 保留 Zellij 的 pane/session 控制；进程并发启动器不能替代窗口管理。开发 codegen 支持独立 watch 和等待后端就绪再 watch。平台差异通过共享工具处理，不依赖 Bash/GNU shell 工具完成正式编排。根 scripts 不增加独立一次性 sh/py/mjs 入口；实验资料保持本地忽略。

## TypeScript 产物与应用生命周期

| 目录 | 用途 |
| --- | --- |
| `dist-tsc/{{project}}/` | tsc 项目引用中间声明与 declaration map，不作为包入口 |
| `dist/` | Vite 应用与 tsdown 的 JS、对外声明 |
| `dist-tsc/{{project}}.tsbuildinfo` | 增量元数据 |

recorder bindings 由 ts-rs 生成，保留生成器原始格式，排除 Biome 的 lint、formatter 和 import 整理。生成一致性通过文件集合/内容哈希检查，类型与声明正确性继续由 tsc/tsdown 检查；不要在 Rust 测试或导出后再次格式化。

metadata 指定 TypeScript 根配置，项目成员由该配置的 references 图定义；cleanup/layout 读取同一图。工作区通过 monorepo-tsc 条件直接解析源码，Vite、Vitest、React Email 使用相同条件；外部包入口读取 bundler 的产物。公开导出为声明生成提供明确类型标注。

类型检查使用显式命令：`just check ts` 增量构建引用图，`just types rebuild` 负责清理后的完整门禁构建，`just dev-types` 直接运行原生 `tsc -b --watch`。WebUI 的 Zellij tab 将 Vite 和类型 watch 放在可独立控制的 pane 中。Vite 负责转译与打包；构建和浏览器 fixture 通过 monorepo-tsc 读取源码，可以在声明产物为空时打包。TypeScript 验收阶段负责类型门禁，包含生成的 bindings 与 GraphQL 输入。

增量构建可能留下已删除源码对应的输出；发布阶段清理声明与元数据后强制 rebuild，失败再次清理部分输出。日常 `check ts` 使用增量构建，完整门禁不在强制 rebuild 后重复空跑。清理只删除受管目录；legacy 清理仅删除具有 map/源码证据的生成物，保留手写声明、secrets、数据和回执，禁止 git clean -xdf。

业务时间使用 Temporal/Intl。bootstrap 在应用模块加载前并发检测并按需补齐能力；它处于 polyfill 之前，不实现 Disposable 或访问资源原语。屏障之后的 owner 使用 using/await using 与标准 disposable symbols；多个资源使用原生资源栈，长期所有权通过 move 移交。main 统一负责手写 HMR/pagehide，entry 只创建并返回资源栈；BFCache 的 persisted 页面保留应用。取消启动不挂载应用，清理顺序为 LIFO，即使一项失败也释放其它资源。

能力加载与运行时配置并发执行，各自默认有 10 秒 deadline。失败会取消尚未完成的同伴请求并释放阶段订阅与计时器；重试使用新的请求 signal。不能中止的动态 import 仍可能迟到完成，但其结果不能挂载已取消的应用。dispose 使等待中的阶段静默结束，真实失败进入启动错误页；启动流完成后，应用所有权保留到显式释放。

Node 配置、测试和工具使用原生 `import.meta.dirname` / `import.meta.filename` 定位自身路径，不拼接 file URL 再解码；浏览器业务模块不依赖 Node 的这些属性。[Node ESM](https://nodejs.org/api/esm.html#importmetadirname)

邮件包的 JS/声明由统一 tsdown 配置生成。邮件 playground 使用 Vite 提供开发预览、静态预览站点构建及 HTML/纯文本导出；React Email 组件与渲染器继续负责邮件标记和 CSS 内联，工具链不再使用 React Email CLI/Next 预览站点。`dev-email` 在 loopback:5003 提供模板索引与刷新；`build-email` 输出静态预览到 dist，`build-email export` 输出到 out。预览模块可在默认组件上附加 PreviewProps。[React Email render](https://react.email/docs/utilities/render)

Node 可执行工具与 fixture 使用可擦除的 TypeScript，由受管理的 Node 直接运行。与测试共用的模块导出函数，并通过 import.meta.main 隔离执行副作用。

### 运行时依赖与类型契约

WebUI 只有一个平台边界：`KonobanguEnvironment`，它扩展 SecurityDept SDK environment 而不是复制其模型。平台差异只能表达为传给 `createKonobanguEnvironmentFor{Host}` 的 `CreateKonobanguEnvironmentFor{Host}Options`（浏览器为 `NativeWeb`，单测为 `Test`）；各宿主解析 Konobangu traits 并选择 SDK 宿主栈，再委托给与宿主无关的 `createKonobanguEnvironment`，由后者统一负责 provider 组合、协议 transport 策略、HttpClient 与 SDK router。Konobangu traits 遵循 SDK trait 约定：document、fetch、locale、platform info、media query、UI storage、page visibility、viewport 与资源加密各有一个接口和一个短名注入 token（如 `PLATFORM_INFO`，注册描述为 `"PLATFORM_INFO_TRAIT"`）；时间使用 SDK `TimeTrait`。服务只注入 trait token，不自行读取 `window`、`document`、`navigator` 或 storage 全局对象。不得另建平行的 host、capability 或 ports 抽象，不得在 environment creator 之外增加第二个参数入口或自行拼装 provider 列表。environment 同时是唯一所有者：creator 注册 DestroyRef 与应用 lifetime，构造失败时回滚，并返回一个 Disposable，dispose 即结束应用；不存在单独的 runtime 对象。

`KonobanguBootstrapEnvironment` 是 polyfill 之前更小范围的环境：document origin、不带凭证的 `BootstrapConfigClient`、polyfill 屏障、阶段期限与 loopback 策略，由 `createKonobanguBootstrapEnvironmentFor{Host}` 创建且不导入 SDK。原生 traits 用 SDK SyncStorageTrait 提供 UI 偏好／临时存储与命名空间事件；权限拒绝时，数据和事件一起绑定同一 SDK memory store。UI storage 不启用认证 token 持久化。环境不满足调用方需要时，通过 creator options、`providers`（可覆盖任意单元）或外层包装 creator 扩展，不重建组合。认证接入见[认证契约](002-AUTHENTICATION-DECISION.md#依赖与类型边界)。

外部值在运行时 schema 校验前保持 unknown。表单草稿、已验证业务数据与 mutation input 使用不同类型。Context 缺 provider 时为 null，由消费 hook 明确检查；错误属性必须检查实际类型。callback wrapper 只承诺参数／返回值签名，不承诺原函数任意附加属性或重载。不要用双重断言掩盖漏字段、未提供能力或不匹配的 mock；受检查 narrowing、schema 校验后的边界适配与字面量推导仍可使用。

### 应用 HTTP 中间件

业务服务使用唯一、经 schema 校验的 HttpClient。Ky 2.1.0 提供内部 HTTP 执行器，项目 middleware 统一负责目的地、caller/root/identity 取消、显式 Bearer 授权、omit 凭据、状态处理与允许的恢复；Basic 只属于内置 Provider 登录。SDK 协议 transport 保持独立，discovery/token 交换不能递归进入应用认证。native fetch、Ky 与 SDK transport 是实现细节，不是领域依赖。[Ky](https://github.com/sindresorhus/ky/tree/v2.1.0)

默认不重试。明确可重放的操作最多在同一身份 scope 内重试一次；凭据检查命令和 mutation 不自动重放。profile probe 的 401 不递归刷新自身。JSON 所有权覆盖解析完成，stream 所有权覆盖 body 完成或取消。Apollo/cache 仍归身份 scope，HTTP 策略统一复用。

### Disposable 所有权

释放消耗唯一所有权。资源成功获取后恰好登记一次到私有 owner，消费者只取得不含释放方法的借用接口。lazy provider 在创建时登记，不在释放时登记；dispose 不解析未使用 provider。局部构造栈在失败时回滚，成功后显式 move 所有权。

bootstrap coordinator 可去重 HMR/pagehide/stop 请求，但底层应用 lifetime 只释放一次。业务资源不以 started/disposed 守卫静默容忍重复 dispose/setup；必要诊断放 owner 层，非法登记或释放 fail-fast。epoch、single-flight、cancel/stop 是独立的业务／并发协议。

root stack 仅拥有一个 SDK DestroyRef，后者拥有 TokenSet registry；OIDC record/client 仅由 registry 拥有。应用资源另有唯一 root-stack 登记，依赖先于消费者获取，LIFO 先结束 UI、query/request，再结束其依赖的认证状态和 SDK。React provider 借用 injector，不取得根释放权。身份变化替换私有 scope，不重挂整个 RouterProvider。shutdown 真实取消在途操作，待完成 logout/初始化不能在 root 结束后重新注册 client。

### 领域对象与 React 桥接

领域状态存在于不依赖框架的服务与对象中，React 只负责桥接。

- **领域拥有自己的生命周期。** 何时开始、续期、重试、重置或取消，由脱离 React 也能工作的领域服务或对象决定。组件不在 effect 里执行这些逻辑，也不用 `useState`/`useRef` 保存领域状态。
- **服务只通过自己的对象驱动 React。** 服务可以响应上游领域的 signal 来重置自己，但对外发布的是自己的 signal，不转述上游领域的规则。认证服务发布不含同身份续期的身份变化；GraphQL 服务收到后停用自己的 Apollo client 并发布新的 client，授权仍由 HTTP client 负责。
- **视图范围的领域对象使用 connect，不使用 dispose。** 视图在一段时间内使用的对象暴露状态 signal 与计数、可重连的 `connect()`/`disconnect()`，Strict Mode 或 Suspense 反复挂载视图不会终结它。对象由所属服务创建，释放权仍只属于该服务。只有当共享能带来服务尚未提供的好处时，才为这类对象引入身份或注册表。
- **桥接层保持很薄。** provider 或组件只读取 signal、连接与断开、转发用户意图并上报元素层面的事实（靠近视口、加载完成、加载失败）。`AuthProvider` 与 `GraphqlProvider` 在根部桥接各自领域；`PrivateIdentityView` 重挂载没有服务可以重置的组件本地状态；`ResourceLink` 与 `Img` 连接资源领域的 `ResourceBinding` 与 `ImageBinding`；signed-out 页面连接认证领域的 `SignInFlow`。
- **与路由绑定的 CRUD 页面例外。** 数据与表单的生命周期恰好等于其路由的页面，直接使用 TanStack Query/Form 与 Apollo hooks；在那里再加领域层只是重复路由的生命周期。
- **每个领域对象都有不依赖框架的测试**，位于其服务旁；组件测试只覆盖桥接。

### 测试组织

每个 apps/packages 项目用目录区分测试层级：

| 层级 | 位置 | 发现规则 |
| --- | --- | --- |
| JS/Node/TS/Python 单测 | 源码邻近的 `**/__test__/**` 或项目根 `tests/**`，排除 integration/e2e | `*.test.*` 或 `*.spec.*`，使用语言实际扩展名 |
| Rust 单测 | 内联 `#[cfg(test)]` 模块或独立测试模块 | Cargo 原生 lib/bin 单测发现机制 |
| 集成测试 | `tests/integration/**` | `*.test.{ts,rs,py}` 或 `*.spec.{ts,rs,py}`，以及适用的 JS/TS 扩展名 |
| E2E | `tests/e2e/**` | 同样的文件名约定，覆盖完整产品流程 |

JS/TS 扩展名包含 js/ts、jsx/tsx 及 c/m 模块变体。Runner 使用 glob 花括号，不使用正则的括号或竖线。目录声明层级，仅使用浏览器不意味着 E2E。单测可使用受控临时文件或子进程 fixture；集成测试组合生产组件或外部服务；E2E 验证完整用户流程。辅助模块、fixture 和快照保留在所属层级旁，不作为测试入口；共享 fixture 可供多个 suite 使用。

WebUI 单测只能通过测试宿主取得应用：`createKonobanguEnvironmentForTest` 把 SDK 测试环境与确定性的 Konobangu traits 组合起来，共享 support 夹具在其上包装请求 handler、可控 principal、provider 渲染与延迟应答，并在测试结束时统一释放。某项行为由拥有它的 suite 单点断言；其它 suite 需要该行为时使用共享夹具或通过 `providers` 覆盖单个单元，不重新手写 mock、provider 列表、injector 或清理登记。spy、全局对象、定时器与 React 的清理属于全局 setup，不写在各文件里。

仓库工具模块在 scripts 下遵循同样布局：单模块契约位于 scripts/tests，真实 Just/CLI 组合位于 scripts/tests/integration，现有 tooling runner 递归收集两者。Cargo 显式注册嵌套集成测试入口并保留原 target 名，辅助模块和 E2E 支持 binary 不作为测试 target。Vitest 只收集源码旁的 __test__ 目录，WebUI 根目录的 tests 完全归 Playwright；应用的构建期模块（如 Vite 插件）放在该应用的 `tooling/` 目录并自带 `__test__`，由其 scripts TypeScript 项目而非应用项目做类型检查；Playwright 仅按层级设置 integration/e2e 两个项目，分别收集对应目录，互不重叠；spec 文件直接位于所属层级，共享的启动器放在 support 下，不按功能目录单独设置项目。依赖完整部署的 e2e spec 自行导入自动部署 fixture，并在 suite 内声明同文件用例顺序运行，共享该 worker 的后端；其他 suite 保持并行，列出用例不获取服务。继续用既有 runner 的原生过滤器筛选用例，不新增 recipe 或 CI 门禁步骤。

Python 包目前没有测试；已配置 pytest 收集 *.test.py/*.spec.py，并使用 importlib 模式支持带点文件名及不同 suite 中同名模块，未引入空测试门禁或新依赖。[pytest 发现规则](https://docs.pytest.org/en/stable/example/pythoncollection.html#changing-naming-conventions)、[pytest 导入模式](https://docs.pytest.org/en/stable/explanation/pythonpath.html#import-modes)

本次重构后的首次发布前，直接修改已有迁移定义；定义改变后重建隔离测试数据库，中间实现不增加兼容迁移。

## 平台制品与发布

`release.artifacts` 独立选择 bundle、运行镜像和 testing-torrents。通过 `just release plan` 查看源码/配置哈希与选择，`--format github-output` 将同一数据提供给 CI；空选择只验证、不发布。这些开关不裁剪测试，也不授予权限。Bundle 发布检查回执中的 metadata 哈希/选择；镜像发布与最终 manifest 遵循[发布交付契约](005-RELEASE-ACCEPTANCE.md#metadata-选择与发布历史)。

JPXL、静态 WebP 和 aws-lc 保留真实原生构建前置条件，codec 源码由上游维护并锁定 Git commit，不嵌入源码树；不设置 target-cpu=native。Windows 使用静态 CRT；musl 制品无 ELF interpreter/NEEDED；GNU 制品记录 glibc ABI 边界；macOS 仅允许系统 dylib/framework。各目标共用 build-release/platform-check 接口；已有制品可以直接检查，无需再次构建。媒体功能和并行语义见 [媒体与配置](004-MEDIA-AND-CONFIGURATION.md)。

`konobangu-metadata.toml` 管理版本、包清单和发行路径。版本设置先验证全部 manifest，再同步 Rust/Node/Python 与锁文件；失败恢复文件，不提交、推送或创建 tag。项目为应用，Node 包 private、Rust 包 publish=false。

```text
mise exec -- just release plan
mise exec -- just release version set 0.2.0
mise exec -- just release prepare --target native
```

本地 prepare 产出带 local provenance 的预览压缩包，不能作为可公开发布的 CI 输入。首次自动发行前，先让 `release.yaml` 进入默认分支，使 GitHub 能够 dispatch。将最终版本及 changelog 提交后推送到 release，触发验证与 metadata 选中的发布内容。重试通过 Release workflow 的 source_ref=refs/heads/release、完整 source_sha、tests_run_id 和精确 tests_run_attempt 指定原候选，不读取分支新尖端；稳定版须通过环境审批。唯一 release orchestrator 负责 tag、draft assets、下载核验、完成发布与稳定别名，没有独立本地 tag/upload 命令绕过它。详见[发布契约](005-RELEASE-ACCEPTANCE.md)。配置、数据和 WebUI 仍是运行输入；codec/TLS 不需要额外运行时动态库。

cargo-about 使用锁定依赖生成原生 THIRD-PARTY-NOTICES 并补充 attributions，Vite 从实际 bundled 依赖生成 WebUI notices；通知生成失败阻止打包。不跟踪各版本依赖的许可证副本；musl runtime 补充仅随对应目标交付。平台验收回执记录平台、source/dependency commits、命令和退出码；旧回执不自动适用于新源码。实际执行证据与 CI 配置分开；未完成响应的渐进渲染和模型评估仍见 [roadmap](roadmap/001-SHORT-TERM-ROADMAP.md)。

## 文档与贡献规则

### 文档站点

`apps/docs` 使用稳定版 VitePress 与 Vue。英文约定文档 `README.md`、`CHANGELOG.md`，以及存在时的 CONTRIBUTING/SECURITY/PRIVACY，使用项目根目录无语言后缀的文件作为真源，`docs/en` 通过相对文件符号链接投影。其它语言以 `docs/{lang}` 下同名普通文件为真源；编号主题文档仍放在各语言目录。站点的 `en`、`zh` 目录符号链接投影这些目录，VitePress 将各语言 README 映射到首页、CHANGELOG 映射到更新记录路由。直接修改真源，不单独维护首页内容。GitHub 编辑链接指向实际源码，`public/assets` 投影仓库资源。

开发要求现代操作系统并启用符号链接。Windows 需开启开发者模式或授予创建符号链接权限，克隆时使用 `git -c core.symlinks=true clone ...`；已有检出需启用 `core.symlinks` 并恢复受 Git 管理的链接后再构建。不用副本或 junction 替代。`check docs` 对投影去重，从真源解析相对链接；站点构建还检查投影目标与渲染后的路由。

Just 提供 `dev-docs`（loopback:5004）、`build-docs`（严格检查站内链接，输出至 `apps/docs/dist`）、`docs-preview`；`verify ts` 包含站点构建。VitePress 配置加入根 TypeScript 项目引用图，声明输出至 `apps/docs/dist-tsc`，bundler 临时数据位于 `apps/docs/.cache`。英文路由从 `/` 开始，中文从 `/zh/` 开始；仓库相对链接和配置示例在 GitHub Markdown 与站点中均可使用。

Documentation workflow 在 `master` 上相关内容变更时构建和发布，也可从 `master` 手动触发。在仓库 Settings → Pages 中选择 **GitHub Actions**，设置自定义域名 `konobangu.apeiraco.com`，完成 DNS 验证后启用 HTTPS。该子域名的 DNS CNAME 指向 `apeiraco.github.io`，不附加仓库路径。`apps/docs/public/CNAME`、sitemap 与 canonical 均使用同一域名。PR 通过既有 TypeScript 验证阶段检查，不触发部署。

### 贡献规则

AGENTS.md 维护核心要求和主题索引；当前契约放 docs，未来计划放 roadmap，历史变更放 CHANGELOG，工作交接与回执不作为正式文档依赖。英文与中文用户文档保持双向语言链接，优先链接同语言主题。YAML 使用两空格缩进，代码与注释使用英文。GraphQL 自动字段及 skip keys 遵循 AGENTS.md，变更后运行 codegen 并检查生成输入类型。配置命名、单位与插值见 [媒体与配置](004-MEDIA-AND-CONFIGURATION.md)，迁移/任务交付见 [任务与迁移](003-TASK-DELIVERY-AND-MIGRATION.md)。

[English](../en/001-DEVELOPMENT-VERIFICATION.md) | [中文](001-DEVELOPMENT-VERIFICATION.md)
