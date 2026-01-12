# XMUAssistantBot

**XMUAssistantBot** 是一款专为 **厦门大学转专业交流群** 定制的高性能自动化管理机器人。它基于 Rust 异步生态，通过自研消息路由与分层存储架构，在处理 2000 人规模高并发群聊场景下，实现了极致的 I/O 效率与低内存分配。

------

## ⚡ 性能基准 (Performance Benchmark)

基于项目内 `benches/` 模块的实测数据（环境：Windows 11 x64, Release Mode）：

### 1. 消息路由与分发 (ABI Efficiency)

通过宏优化和精确的命令过滤，大幅减少了不必要的异步任务创建和路由开销。

| 场景 | 核心指标 | 最新耗时 | 性能变化 |
|---|---|---|---|
| 命令命中路由 (`routing_hit_first`) | 核心命令处理前置过滤与路由时长 | `507.21 ns` | 显著优化 |
| 命令未命中路由 (`routing_miss_all`) | 未匹配消息的过滤与退出时长 | `510.28 ns` | 显著优化 |
| 轻量化 Context 克隆 | 精简 Context 结构的复制开销 | `41.39 ns` | (维持超低) |
| 并发 Handler 分发 (x10) | 模拟 10 个 Handler 同时并行运行的系统级开销 | `20.10 µs` | (维持超低) |

### 2. 存储系统性能

采用分层存储策略，并通过定长序列化和优化并发读写来确保性能。

| 场景 | 核心指标 | 最新耗时 | 性能变化 |
|---|---|---|---|
| 冷存储读取命中 (`cold_get_hit`) | 从 Redb 冷存储中读取单个 Key 的耗时 | `11.472 µs` | 改进 |
| 高并发热存储吞吐 (`hottable_concurrent_read_write_x100`) | 100路并发读写 DashMap 耗时 | `49.290 µs` | 改进 |
| 冷存储写入 (`cold_insert`) | 写入 Redb 冷存储耗时 | `2.1858 ms` | 改进 |

### 3. 序列化与协议解析 (Zero-Copy Optimization)

| 场景 | 核心指标 | 最新耗时 | 性能变化 |
|---|---|---|---|
| 零拷贝接收反序列化 (`json_deserialize_message_receive`) | 消息体 JSON 反序列化耗时（LazyString） | `1.1546 µs` | 改进 |
| 消息体序列化 (`json_serialize_message_send`) | 消息体 JSON 序列化耗时 | `357.22 ns` | 改进 |
| 文本段数组获取 (`get_text_array`) | 文本段数组获取耗时 | `93.236 ns` | 改进 |
| 定长文本获取 (`get_text_single`) | 优化后定长字符串获取耗时 | `27.283 ns` | 无变化 |

------

## 🛠️ 深度技术优化 (Technical Deep Dive)

### 1. 极致的内存分配策略

- **MiMalloc 全局分配器**: 放弃系统默认分配器，采用 `MiMalloc` 优化瞬时大量 JSON 对象的分配，极大降低内存碎片率和全局锁竞争。
- **哈希性能**: 引入 `ahash::RandomState` 作为全局哈希状态，以获得更优的哈希性能和更强的抗碰撞能力。
- **零拷贝接收与延迟解析 (LazyString)**:
    - 引入 `LazyString`，在反序列化消息 (`MessageReceive`) 时，字符串字段只记录原始 JSON 字节位置 (`Box<RawValue>`)，**完全避免了初始时的内存分配和字符串转义开销**。只有当 Handler 实际需要使用文本内容时，才会进行解析和物化。
- **栈空间优先与健壮序列化**:
  - 大量使用 `smol_str` 存储短字符串，通过 Inline 优化避免堆分配。
  - **`define_default_type!` 宏**: 确保 API 响应中的可选字段在缺失或为 `null` 时，能健壮地使用默认值。

### 2. 消息流零拷贝与 Send/Receive 彻底分离

项目通过 `ArcWith<T>` 结合 `LazyString` 实现了消息传递的跨线程零拷贝：

- **生命周期绑定 (`ArcWith<T>`)**: 使用 `ArcWith<T>` 结构体将反序列化后的消息体 `T` (包含 `LazyString` 的消息) 与原始的零拷贝数据源 `Utf8Bytes` (来自 WebSocket 层的原始数据) 绑定在一起。
- **内存安全共享**: 这使得 `LazyString` 能够安全地引用底层的 `Utf8Bytes` 数据，并在多个异步 Handler 之间通过 `ArcWith` 共享消息体，**彻底分离了消息的接收和处理生命周期**。
- **避免 Context 拷贝**: 在路由失败时，**彻底跳过 Context 对象的克隆**，结合宏的前置过滤，进一步将 Context Router 性能提升了 **50%+**。

### 3. 基于过程宏的元编程 (Helper Macros)

项目高度依赖自研过程宏来自动化重复代码、确保运行时性能和安全。

- **`#[handler]` / `register_handlers!` 性能与功能增强**:
    - **零成本过滤**: 宏在 `handle` 方法中加入前置同步检查 (`type_filter` 和 `command_filter`)，只有当命中指令时，才调用 `tokio::spawn` 启动异步 Handler 协程。**此优化避免了为不匹配消息创建昂贵的 `tokio::spawn` 协程，显著降低了路由开销**，同时**减少了 Context 对象的克隆**。
    - **自动化 Help 文档**: `#[handler(command = "cmd", help_msg = "描述")]` 现在强制要求提供帮助信息。宏会自动实现 `BuildHelp` trait，结合 `register_handler_with_help!` 宏，**实现了编译时自动聚合所有指令的帮助信息，无需手动维护 Help 命令**。
    - **消息传递优化**: 实现了修改传递 `Uft8Bytes` 替代 `String`，以减少序列化开销。
- **API 框架与客户端抽象**: 针对 OneBot v11 接口进行了彻底的框架重构。使用 `#[api]` 宏配合分离的 `Params` 结构体（`src/abi/message/api/params/`），实现 API 客户端的声明式定义和自动化封装，彻底解耦了不同 API 的实现。
- **新增特殊头衔 API**: 实现了 `set_group_special_title` 接口的封装，支持在群聊中设置特殊头衔。
- **核心 Client 抽象**: 核心 API Client 逻辑（如获取 `SessionClient`）已被重构，从宏中剥离出可复用的 `castgc` 和 `session` 模块，大幅减少了重复代码。`logic/helper` 模块的引入，也为一键获取 `SessionClient` 提供了便利。
- **`#[jw_api]` (教务系统)**: 智能适配教务系统非标准的 JSON 嵌套结构，**现已支持配置 `call_type = "GET"` 或 `"POST"`**，以适应教务系统复杂的接口请求方式。
- **教务系统新增路径 (JW Schedule)**: 在 `src/api/xmu_service/jw/` 下新增 `schedule` 模块，提供了包括**课表列表、时间、可读格式**等在内的多项教务系统信息获取路径。

### 4. 异步 I/O 与分层存储架构

项目实现了 **Hot-Cold-File** 语义化分层存储系统：

- **Hot 层 (DashMap)**: 存储当前活跃的会话（`LoginData`），读写复杂度 $O(1)$。
- **Cold 层 (Redb)**: 采用 Copy-on-Write (CoW) 机制的嵌入式数据库，通过异步 Task 批量提交事务，完全剥离主循环 I/O。
- **Hot 层和 Cold 层定长 bincode**: 存储层使用定长的 `bincode` 格式，牺牲少量空间换取更一致、更优秀的序列化/反序列化性能，有效提升 `cold_get_hit` 性能。
- **RAII 临时文件管理**: 自研 `TempFile` 系统，利用 `Drop` 钩子自动触发异步清理，确保存储空间整洁。
- **ABI & WebSocket 健壮性**: Handler 异常屏障在显示错误时会显示出错函数，在 WebSocket 生命周期结束时会自动断连。

### 5. Lnt API 深度集成

- **Lnt API 深度集成**: 实现了包括 `activities`、`file_url`、`my_courses`、`profile` 修正等在内的 LNT API 封装，并**新增了考试查询、成绩与试题分发等关键接口**：`distribute` (获取试题)、`exams` (考试列表)、`submissions` (提交记录) 和 `submission_id` (查询答案)。

### 6. LLM 工具驱动

- **`#[derive(LlmPrompt)]`**:
    - **LLM 工具描述生成**: 自动从 Rust 结构体和自定义类型（`tool/type` 下的 `LlmVec`, `LlmOption` 等）中提取信息，生成 LLM 函数调用所需的精确 Schema。该自定义类型系统是为了解决模型返回格式不精确的问题，**实现了实测高准确率的工具化调用**。

### 7. 其他核心优化

- **编译优化**: 升级了编译优化配置，生成效率更高的二进制文件。
- **智能 JSON**: 提供了更加智能的 JSON 解析函数，在 Release 模式下等效于原本的快速 JSON 函数，同时提升了开发和报错体验。

### 8. Expose 模块：带上下文的文件暴露

- **目标**: 弥补 OneBot v11 等消息平台对大文件/多文件的支持不足。
- **流式下载与零拷贝**: 基于 `axum` 的 `ReaderStream` 实现从磁盘到浏览器的零拷贝直传。
- **任务级缓存与状态**: 支持任务处理状态显示，文件链接默认设置 **24 小时** 过期时间，并修复了任务 URL 上的文件名显示问题。

------

## 🤖 LLM 驱动的工具调用 (LLM Tool Calling)

本项目内置了 LLM 工具链，允许模型通过自然语言指令直接调用复杂的厦大服务 API，以实现智能化操作。这些工具的描述和调用模式通过自定义宏自动生成。

| **工具名称** | **功能描述** | **调用场景** |
|---|---|---|
| `ChooseCourse` | 智能查询/筛选/匹配教务系统中的课程信息。 | “帮我查一下大三上学期的微积分课表。” |
| `ChooseFiles` | 基于用户会话和课程信息，智能定位学习通文件。 | “帮我把高数最近的作业资料下载一下。” |

------

## 💬 指令概览 (Commands)

| **指令** | **逻辑流** | **优化亮点** |
|---|---|---|
| **`/login`** | 身份认证 -> 持久化 | 自动维护教务会话，多端登录不冲突。**新增 LNT 失败回退 JW 机制**，保证登录会话获取的健壮性。 |
| **`/download`** | 触发 Expose -> 返回链接 | 支持文件分块下载，链接 24h 有效 |
| **`/logout`** | 缓存注销 -> 状态回滚 | $O(1)$ 时间复杂度快速清理热数据 |
| **`/help`** | 自动聚合所有指令帮助信息 | **编译时生成，无需手动维护**。 |
| **`/echo`** | 消息回传 | 继承 `is_echo` 标记，用于端到端延迟测试和指令流判断 |

------

## 📁 项目结构 (Project Structure)

Plaintext

```
├── helper/         # 过程宏定义：实现元编程与 API 自动化封装
├── src/abi/        # 自研 ABI 层：包含路由、消息体解析、网络适配，以及重构后的 OneBot v11 API 声明
├── src/api/        # 核心业务接口：存储(Storage)、教务服务(XMU Service)、LLM
├── src/logic/      # 业务逻辑实现：登录流程、下载处理、客户端辅助
├── src/web/        # Web 服务：Expose 文件暴露系统，弥补文件传输不足
└── benches/        # 性能基准测试：压榨每一纳秒的运行效率
```

------

## 🤝 参与贡献 (Contributing)

我们非常欢迎来自厦大或其他高校的开发者提交 PR！

- **优化 API 定义**: 如果你发现了教务系统（Jw）或畅课系统（Lnt）的新接口，欢迎补充宏定义。
- **改进 Web 视图**: 期待你对 `Expose` 模块 HTML 界面的 UI/UX 优化。
- **性能提升**: 欢迎针对 `benches/` 中的指标提交更优的算法实现。

**提交规范**:

1. 代码必须通过 `cargo fmt` 格式化。
2. 高性能模块的修改请附带 `cargo bench` 结果对比。
