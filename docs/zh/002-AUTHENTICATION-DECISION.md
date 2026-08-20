# 002 — 认证与数据访问

## 认证契约与部署边界

WebUI 与 API 可独立部署，包括互不相关的注册域名。内置与外部 issuer 使用同一模型：SecurityDept 0.3.3 frontend-oidc TokenSet client 完成公开客户端 Authorization Code + PKCE S256，Recorder API 验证 JWT Bearer access token。静态资源公开；业务 API 不接受 session cookie 或 Basic 凭据。图片和 RSS 还可使用下述受限原生资源凭证。参见[发布验收](005-RELEASE-ACCEPTANCE.md)。

内置 issuer 属于独立 Rust crate `packages/auth-provider`。SecurityDept Basic 只用于专用 `/auth-provider/login`，由浏览器原生凭据验证用户；WebUI 没有凭据表单、凭据存储、Basic SDK context 或 iframe 登录适配。Provider 登录后向精确登记的回调返回授权码，业务 API 仍统一使用 Bearer。外部 issuer 直接接入，不经过额外 broker。

API audience `urn:konobangu:api` 与必需 scope `konobangu:api` 独立于浏览器 client ID。资源验证检查签名、issuer、audience、过期／生效时间、必需 scope 和非空 subject，并要求非对称算法及 `typ=at+jwt` 或 `application/at+jwt`。ID token、refresh token 和 Basic 不能调用 API。外部 issuer 复用 SecurityDept discovery/JWKS 缓存；内置 runtime 直接提供公钥，避免 listener 启动前通过 HTTP 自发现。无需每请求访问 IdP 或应用会话表。

应用身份仅来自 `GET /api/auth/user-info` 的服务器验证投影，浏览器 ID claims 和 Provider userinfo 不代替 subscriber 映射。principal 必须具有非空 issuer、subject 与 `authType: "oidc"`，私有请求／缓存由已验证的 `(issuer, subject)` 身份拥有。替换 SDK material 会取消旧确认，晚到结果不能授权新 material；同一身份的正常续期保留页面草稿和 Apollo cache，但受保护请求必须等待新 material 的服务器确认。发送前再次检查借用的授权，principal probe 绕过此门禁避免递归；不另建 token store。

### 运行时部署配置

同一预构建 bundle 在 origin 根读取 `konobangu.config.json`，例如 `{"version":1,"apiBaseUrl":"https://api.example.net/api/"}`；默认 API 路径为 `/api/`，也允许反代前缀。拒绝用户信息、query、fragment、协议相对地址和非 HTTP(S) URL。HTTP WebUI 可以显式选择 HTTP API；HTTPS 页面仍受 mixed-content 限制。缺失或非法配置直接报错，不猜测凭据目的地。该文件不携带 token、secret 或认证模式。

公开 `GET <apiBaseUrl>auth/config` 响应 `no-store`，不创建 cookie/session、不访问身份库。它始终返回 `provider: "oidc"`、resource envelope 与平铺的 SecurityDept `FrontendOidcModeConfigProjection`；浏览器不知道服务端的 `bundled`／`oidc` 私有选择。SDK 校验其结构，宿主额外拒绝 client secret、可执行 claims script、关闭 PKCE、空标识，以及不属于当前 WebUI `/auth/callback` 的回调。

`auth.webui.allowed_origins` 使用严格 origin 根或完整匹配 regex，空／无限制规则启动失败。公开配置从 Origin 选择 WebUI，仅 Origin 缺失时使用 Referer；非法、重复、null Origin 不能回退。Host、Forwarded、XFF 和 real IP 不决定 WebUI 或 issuer。响应 `no-store`，按 Origin/Referer vary。每个 OAuth callback 还必须独立登记，WebUI regex 不等价于 callback 注册。

### 依赖与类型边界

`createKonobanguEnvironmentForNativeWeb` 创建唯一 environment 与 DI 根，参见[开发规则](001-DEVELOPMENT-VERIFICATION.md#运行时依赖与类型契约)。polyfill 前由 `KonobanguBootstrapEnvironment` 读取公开 JSON，不提前加载 SDK。领域使用共享 HttpClient 和构造好的具体 trait，不消费原始宿主输入或 SDK transport。外部 JSON 在 schema 校验前保持 unknown；SDK 自有协议结构由 SDK schema 管理，边界适配不构成双重断言或残缺 mock 的理由。

## 内置 Provider

issuer 是固定配置的 HTTPS URL，以 `/auth-provider` 结尾，可带部署前缀；仅开发／测试允许回环 HTTP。公开端点从该 URL 生成，不读取 Host/Referer/XFF，反代必须转发对应路径。LAN HTTP WebUI/API 可共用该 HTTPS issuer。仅支持一个预登记 public client、`response_type=code`、PKCE S256、authorization-code/refresh grant 与 `token_endpoint_auth_method=none`。

| issuer 下端点 | 职责 |
| --- | --- |
| `/.well-known/openid-configuration` | OIDC discovery metadata |
| `/authorize` | 登录前校验 client、精确 callback、scope、response type 和 PKCE |
| `/login` | 唯一 Basic challenge／凭据端点，恢复短期已验证 pending 请求 |
| `/token` | public client 授权码兑换与 opaque refresh token 轮换 |
| `/jwks` | 仅公开 RSA 公钥 |
| `/userinfo` | Bearer 保护的 OIDC subject 信息 |

OAuth2 状态转换复用 `oxide-auth` 与 `oxide-auth-async`，`openidconnect` 构造 typed OIDC metadata 和 ID token，Basic 与 real-IP 使用 SecurityDept。real-IP 默认仅信任 transport peer；可信代理设置不影响 issuer/callback。无 password grant、implicit、动态注册、通用账号／consent UI、登录 session、introspection 或 access-token 撤销。

pending、授权码和 refresh grant 位于 `auth_provider` schema，由独立受限能力连接池访问，秘密只保存散列。授权码有期限且原子消费，绑定 client、callback、subject、scope、PKCE、nonce/auth_time。refresh 轮换在事务内消费旧记录并创建后继；重用使同一 family 后续不能刷新，不影响已签 JWT 的有效期。过期协议记录按有界批次清理，不新建调度子系统。

生产必须配置持久化、至少 2048 bits 的 RSA 私钥，支持文件 secret。JWKS 不包含私钥，kid 稳定。access token 使用 RS256 和 RFC9068 claims，ID token audience 为浏览器 client，保留 nonce/at_hash 语义。缺失／非法密钥启动失败，不自动生成临时 Provider 身份。

## 原生资源凭证

`POST /api/auth/resource-grants` 要求 Bearer 与 JSON `{}`，返回 `grant`（Macaroon V2JSON）、RFC3339 `issuedAt`／`expiresAt` 和 `childTtlSeconds: 300`。内存一级凭证绑定服务器确认的 subscriber、`native-resource:read`、Recorder audience 和 3600 秒期限，不能授权 API、自我续期或直接读取资源。原生加载前 ResourceService 使用 HMAC-SHA256 在本地衰减为精确图片／feed 路径及不超过 300 秒的父期限内窗口。

仅登记的 GET/HEAD 图片与 RSS 端点接受 `resource_token` query。URL 只含二级签名，不含父签名或 OIDC access token；所有限制在数据库／存储访问、validator 和 304 前验证。资源标识基于解码逻辑路径，与域名／反代前缀无关；拒绝 Bearer 与 query 凭据同时使用，不保留 Basic 兼容分支。图片通过匿名 CORS 原生加载，无 Service Worker、PWA 或手工图片／RSS fetch。

唯一自有资源服务合并一级兑换，在存在活跃可见资源时提前五分钟续期，隐藏／不活跃页面停止定时器。网络／超时／5xx 最多按 5/10/20 秒重试三次，401/403/schema 错误不定时重试。常规 token／一级续期不替换已加载图片 URL。原生图片失败允许一次强制续期重试，先清空旧 src 再重挂元素，即使 URL 相同也只产生一次替换请求。身份变化取消获取并清理父凭据；本地退出不撤销已签二级凭据。

RSS 二级凭证绑定 `feeds/rss/<locator>`，locator 本身不是认证。owner RLS 继续拒绝访问其他 subscriber 的 feed。响应 XML、`no-store`、`Referrer-Policy: no-referrer`，self-link 无 query 凭据。ResourceLink 通过 connectable binding 渲染原生链接，可见时本地续期，隐藏／身份变化时清理陈旧 URL。这些短 URL 用于浏览器打开，不是永久外部订阅凭据；外部阅读器需自带 Bearer，机器凭据仍属未来能力。

`auth.resource_grants.signing_key` 为 32 字节无填充 base64url，也支持 `AUTH__RESOURCE_GRANTS__SIGNING_KEY_FILE`。省略时生成临时资源密钥；副本若需重启／多实例连续验证，应配置相同显式秘密。该密钥独立于 Provider RSA key。密钥、凭据及 query token 脱敏，包括 macaroon 的敏感 TRACE 元数据。

### Crypto bootstrap 与 LAN HTTP

并发 capability barrier 保留原生 Web Crypto，在 SDK 构造前通过 `@noble/hashes` 动态补齐缺少的 SHA-256/HMAC。原生 `getRandomValues` 必须可用，不关闭 PKCE、不伪造安全上下文。LAN HTTP WebUI/API 仍连接 HTTPS issuer，并精确登记 LAN callback。计算 polyfill 不提供传输机密性，也不绕过 mixed-content、CORS 或 IdP 策略；反代日志避免保存 query 凭据。

## 凭据检查与系统计划

`POST /api/credential3rd/{id}/check-available` 在 Bearer 下接收 JSON `{}`，返回 `{ "available": true }`，不缓存。短 owner 事务读凭据与 PostgreSQL `xmin` 后关闭，再执行远程检查，最后以新事务按同 owner/ID/version 条件写入加密 cookie。并发编辑／删除／检查返回 409，不写陈旧结果；跨 owner 返回 403，远程失败返回 502。重试须重读当前数据，回滚不能撤回远端已收到的请求。

系统任务和系统 Cron 使用独立于认证方式的显式应用管理员能力。内置管理员由可信配置 issuer 和固定 subject 确认，修改 username 不改变身份；外部管理员仅由显式可信 issuer/subject 列表授予，默认没有管理员。历史共享 subscriber 或 token 自报 role/email 不授予权限。非管理员 root/relation/update/delete/batch 继续组合 subscriber scope 与 `system_task_cron IS NULL`。参见[任务交付](003-TASK-DELIVERY-AND-MIGRATION.md)。

## 配置与角色

Figment 按默认值 < TOML/JSON/YAML < canonical 双下划线变量加载，进程环境优先于 dotenv。结构化值使用 Figment 的 TOML-like 语法，例如 Just/dotenvy 文件：

```dotenv
AUTH__WEBUI__ALLOWED_ORIGINS='[{type="strict",value="https://ui.example"}]'
AUTH__PROVIDER__CREDENTIALS__PASSWORD_FILE=/run/secrets/provider-password
AUTH__PROVIDER__SIGNING__PRIVATE_KEY_PEM_FILE=/run/secrets/provider-key
```

外层引号属于 dotenv loader，Docker `--env-file` 会将其视为字面值，使用该 loader 时省略外层引号；复杂列表留在配置文件。参见[配置来源与秘密](004-MEDIA-AND-CONFIGURATION.md#配置来源)。

```toml
[database]
url = "postgres://konobangu:PASSWORD@localhost/konobangu"

[auth]
# External administrators only; use exact verified issuer and subject.
# administrators = [{ issuer = "https://idp.example/", subject = "operator-id" }]

[auth.webui]
allowed_origins = [{ type = "strict", value = "https://ui.example" }]

[auth.provider]
type = "bundled"
issuer = "https://api.example/auth-provider"

[auth.provider.client]
client_id = "konobangu-webui"
redirect_uris = ["https://ui.example/auth/callback"]
scopes = ["openid", "profile", "konobangu:api", "offline_access"]

[auth.provider.credentials]
username = "admin"
# subject defaults to stable "local-admin".

[auth.provider.tokens]
authorization_code_ttl_seconds = 60
access_token_ttl_seconds = 300
refresh_token_ttl_seconds = 2592000
```

使用 Node 生成私钥，无需外部 OpenSSL executable，文件不得提交到仓库：

```text
node -e "const fs=require('node:fs'); const c=require('node:crypto'); fs.mkdirSync('secrets',{recursive:true}); fs.writeFileSync('secrets/provider-key.pem',c.generateKeyPairSync('rsa',{modulusLength:2048,privateKeyEncoding:{type:'pkcs8',format:'pem'},publicKeyEncoding:{type:'spki',format:'pem'}}).privateKey,{mode:0o600,flag:'wx'})"
```

接入外部 issuer 时替换 provider 整组配置：

```toml
[auth.provider]
type = "oidc"
issuer = "https://idp.example/"
# trusted_ca_file = "/etc/konobangu/idp-ca.pem"

[auth.provider.browser]
client_id = "konobangu-webui"
scopes = ["openid", "profile", "konobangu:api", "offline_access"]
```

登记 auth method 为 none、PKCE S256 的 public code client 及全部精确回调，并将其 JWT access token 配为 `urn:konobangu:api` 与 `konobangu:api`；SDK 0.3.3 不发送 RFC8707 resource 参数。两种来源都使用 `auth.provider.resource.audience/required_scopes`。私有 CA issuer 必须提供 API／浏览器有效信任，外部模式支持 `trusted_ca_file`。discovery/configuration 失败直接报错，不回退 Basic 或开发身份。

运行权限使用固定 NOLOGIN、NOINHERIT、NOSUPERUSER、NOBYPASSRLS 能力：

| 角色 | 职责 |
| --- | --- |
| `konobangu_app_scoped_access` | subscriber 约束的应用数据及 worker 业务阶段 |
| `konobangu_auth_access` | 已验证身份映射及 subscriber 初始化 |
| `konobangu_task_control_access` | 跨 subscriber 的任务投递、run、outbox、Cron 与队列存储 |
| `konobangu_auth_provider_access` | 内置 Provider 协议状态，不访问应用业务数据 |

默认部署只需 `database.url` 的现有 PostgreSQL login，identity、scheduler URL 继承它；内置 `auth.provider.database.url` 先继承 identity，再继承 database。启动 owner migration 初始化 schema、角色和 grant 后再打开受限 runtime pool。membership 使用 `INHERIT FALSE, SET TRUE`，需要 PostgreSQL 16+、实际 CREATEROLE 与对象 ownership/grant 权限。无需手工 grant SQL，不生成／替换登录密码，失败不关闭 RLS 或回退 owner 查询。

连接建立时激活固定能力，每次 checkout 重置；业务事务额外 SET LOCAL ROLE 并绑定 subscriber，提交／回滚／取消清理事务作用域。Apalis 需要独立受限池，不能继承外层业务事务。使用直连或 session pooling，transaction pooling 无法保持此 baseline。共用 login 是每操作降权，不是对任意 SQL 的凭据隔离，session_user 仍可切换已授予角色。可选独立 runtime URL 与 owner `database.migration.url` 用于凭据隔离，均须指向同库。

`konobangu-database` 拥有共享 pool/role 机制和 migration helpers；Recorder 拥有业务 identifiers 与显式 grant，auth-provider 拥有自身 schema identifiers/migration/grants，owner 入口统一组合，不创建重复历史。runtime pool 不能执行 owner DDL；AuthService、ProviderRuntime 分别唯一拥有并关闭其受限连接池，初始化失败也关闭。

## 已有数据库迁移与恢复

停止 writer／worker，保留经过恢复演练的数据库／存储备份。旧 OIDC issuer 未知或身份冲突时迁移失败，不按 email/display name 猜测；只有明确知道旧 issuer 才设置 `database.migration.legacy_oidc_issuer`。默认启动运行相同 owner migration/grant 路径，显式维护可用：

```sh
mise exec -- cargo run -p recorder --locked --bin migrate-up -- --environment production --config-file /path/to/recorder.config.toml
```

未发布的中间 migration 直接修改，隔离测试库可重建；真实业务数据保留。历史 Basic 行可作为存储记录保留，但不能认证 API。仅可信内置 issuer／固定 subject 可映射历史管理员 seed，新的外部身份拥有独立 subscriber；共享 subscriber 不等于管理员。各组件 DDL 不是一次原子提交，失败时保持停写、修复权限并重跑幂等 owner 入口。恢复使用匹配旧版本代码／配置的备份，不清空生产数据。

## 端点、CSRF 与退出

业务 API 要求显式 Bearer，写入还要求 JSON。OPTIONS 先于认证；CORS 允许配置的 WebUI origin，不允许凭据 cookie，不能替代认证。`/api/login` 不存在，API 错误不发送 Basic challenge；只有 Provider login 接受 Basic。Provider `/token` 的 CORS 只允许登记 callback 的 origin，使用无 cookie／client secret 的 public-client 表单请求。

callback 使用 SDK `useTokenSetFrontendCallback` 和 router 管理初始化、state/PKCE 交换及 CallbackCleanup+Replace 清参；Handled 后只向 SDK 保存的站内目标导航一次，缺失／错误回调不伪造登录。公开退出页启动同一个 SDK redirect，connectable domain flow 拥有 pending/error，卸载不释放应用拥有的 client。

退出清理本地 SDK material/principal，取消私有请求并结束旧 Apollo 身份；不为退出初始化 lazy client，只对 ready client clearState，再 unregister 让 registry 释放并注册新 lazy entry。不调用 revocation/introspection，已签 access/resource token 有效至期限。外部 SSO 或内置 Basic 浏览器缓存可能保留，后续登录不保证再次提示凭据。仅短期 redirect state 位于 sessionStorage，token/projection 只在内存。

共享 HttpClient 归一化请求，校验目的地与 epoch，始终 omit credentials；受保护请求要求 registry Bearer，公开配置不初始化认证。仅显式 safe 操作允许一次有界重放，principal probe 和 mutation／凭据命令不递归刷新或自动重放；SDK discovery/token transport 独立于业务恢复。仅 Provider 确认 invalid grant 才清理续期状态，网络／403 仍是错误。

## GraphQL、任务与前端所有权

在线执行使用显式 async-graphql dynamic resolver，Seaography 仅提供 schema/naming/filter helper。每个 operation 拥有 subscriber 事务，由 root/relation/loader/CRUD/batch/task mutation 共用；resolver 不提交，loader 不跨请求缓存，mutation 提交成功后才响应。执行失败回滚并返回 data=null；查询 nullable 边界保留部分数据。取消丢弃未提交事务，缺失／空 subscriber scope 拒绝私有行。

owner 只来自验证映射，不接受前端 subscriber_id；insert 注入 owner，update 不改 owner。因 PostgreSQL FK 检查可能绕过 RLS，引用可见性在同 owner 事务检查。introspection 排除混合文档的业务执行。系统操作使用显式管理员，任务在远程／图片工作前关闭短业务事务，再按[任务契约](003-TASK-DELIVERY-AND-MIGRATION.md)提交 upsert/outbox/fenced 阶段。

SDK registry 拥有 client/factory，AuthService 协调身份，ResourceService 拥有资源凭证；environment 拥有生命周期，React 借用。每个身份拥有一个 ApolloClient/cache，身份变化／退出取消旧 client 并拒绝晚到结果，同身份刷新保留它。query/watchQuery 可显示部分数据和错误，mutation 错误不算成功写入；HTTP 403／网络／5xx／解析错误不触发登录循环。

[English](../en/002-AUTHENTICATION-DECISION.md) | [中文](002-AUTHENTICATION-DECISION.md)
