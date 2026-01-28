# Bevy 体素 MMO：Chunk 域模拟与模块化设计纲领

> 目标：在 **Bevy(ECS)** 上构建可持久化、可扩展的体素世界模拟。用 **Chunk 作为数据与调度粒度**，将“温度/湿度/燃烧/相变/腐蚀/生长…”等多条“物理线/领域线”以可组合的方式逐步加入，避免全图扫描、避免域间强耦合，并兼容 **MMO 的存档与网络增量同步**。

---

## 0. 核心结论

- **方块（voxel/block）不是 ECS Entity**：它是 Chunk 内部数组的一项。
- **Chunk 是 ECS Entity**：一个 Chunk Entity 承载该区域的所有体素数据（类型、状态场、活跃集合、dirty 标记、diff 日志）。
- **一条领域线 = 一个 Domain 模块**：提供域状态（放 Chunk 里）、域系统（FixedUpdate 内跑）、域 API（封装访问）。
- **跨域交互统一走 Reaction/Commit 管线**：域不直接写别的域内部结构，避免耦合爆炸。
- **Bevy ECS 负责调度与资源装配**；核心模拟尽量保持 **纯 Rust（Core）**，便于服务端、回放、测试与后续替换。

---

## 1. 概念与数据粒度

### 1.1 Chunk 的定义与坐标
- 世界坐标拆分为：
  - `chunk_coord = floor(world_pos / CHUNK_SIZE)`
  - `local_pos = world_pos % CHUNK_SIZE`
- Chunk 尺寸推荐从 `16×16×16` 起步。

### 1.2 Chunk 与方块的关系
- 一个 Chunk 包含 `N = CHUNK_SIZE^3` 个格子。
- 每个格子用线性索引 `idx` 表示：
  - `idx = x + S * (y + S * z)`，其中 `S=CHUNK_SIZE`
- 方块类型与状态：
  - `blocks[idx] : BlockId`（每格一个，描述“是什么”）
  - 动态状态（温度、湿度、燃烧状态…）以并行数组 / 稀疏表存储（描述“怎么样”）

---

## 2. 总体架构分层

> 推荐：**Core（纯 Rust）** + **Bevy 集成层** + **Client/Server 组装层**。

### 2.1 Core（不依赖 Bevy）
包含：
- `block/`：`BlockId`、`BlockDef`、注册表（静态材质属性）
- `chunk/`：Chunk 数据结构、坐标/索引转换、读写 API
- `domains/`：各领域（thermal、moisture、combustion、phase_change…）
- `reaction/`：规则引擎（条件→效果），或最初的硬编码规则集合
- `commit/`：统一提交命令（唯一写回入口）
- `diff/`：变更日志（存档/网络增量同步格式）
- `sim/`：固定步长 tick 管线定义（可在服务端/测试中直接跑）

### 2.2 Bevy 集成层（world_bevy）
包含：
- `ChunkPlugin`：Chunk Entity 生命周期、加载卸载、资源索引
- `SimPlugin`：FixedUpdate 调度、SystemSet 顺序、事件/命令队列清理
- `Domain Plugins`：把各领域系统注册进指定 SystemSet
- `NetPlugin`：服务端打包 diff、客户端应用 diff
- `Client only`：`MeshPlugin`、`VfxPlugin`（网格重建、表现系统）

### 2.3 Server / Client 组装
- 服务端：Chunk + Domains + Reaction/Commit + Net（无渲染）
- 客户端：Chunk + Net + Mesh/Vfx（可选本地预测）

---

## 3. 数据模型：类型、通用状态、专用状态

### 3.1 方块类型（BlockId + BlockDef）
- `blocks[idx] = BlockId`
- `BlockDef[BlockId]` 提供静态属性（不重复存）：
  - 热学：热容、导热系数、环境交换系数
  - 可燃：`is_flammable`、`ignition_temp`、`burn_energy`、`burn_rate`、`char_block`、`ash_block`
  - 相变：`melting_point`、`freezing_point`、`liquid_to_solid_block` …
  - 其他：密度、强度、腐蚀敏感度、生长参数等

### 3.2 通用状态（轻量，全量）
- `flags[idx] : bitflags`（例如 Burning/Frozen/Wet/Charred/Damaged…）
- `variant[idx] : u8`（阶段/层级：水位、结霜层数、植物生长阶段等）

### 3.3 专用状态（按需，稀疏/延迟分配）
用于“只有少量格子会激活”的领域状态：
- `SparseMap<idx, State>` 或 `Option<Vec<State>>`（按需分配全量数组）
- 例如：
  - 燃烧：`burn_state[idx] = {fuel_left, intensity, ...}`
  - 温度/湿度场：`temp_overrides`, `moist_overrides`
  - 腐蚀：`corrosion_level[idx]`（稀疏即可）

> MMO 建议优先：**默认值 + 稀疏覆盖** 或 **按需分配**，避免每个 chunk 都背全量状态场。

---

## 4. 性能关键：活跃集合 + 脏标记 + 局部处理

### 4.1 活跃集合（避免全量扫描）
每个领域维护自己的活跃索引集合：
- `active_burning`、`active_thermal`、`active_freezing`、`active_flowing` …

系统只处理活跃集合里的 idx：
- 点燃判定：只看 `active_thermal`
- 燃烧 tick：只看 `active_burning`
- 相变：只看温度变化影响到的 idx

### 4.2 脏标记（渲染/碰撞/网络）
- `dirty_blocks: Vec<idx>` 或 bitset
- `needs_remesh: bool`（或分级：mesh/collider/navmesh）
- `changes: Vec<ChangeOp>`（diff 日志）

---

## 5. 统一执行管线：SystemSet 与 tick 阶段

> 必须固定顺序，防止“系统互相打架”。

推荐每个 FixedUpdate tick：

1. **ExternalActions**
   - 玩家/脚本输入 → 生成请求（Ignite/Extinguish/AddHeat/SetBlock…）

2. **FieldUpdate**
   - 连续场更新：热扩散、湿度扩散、冷却/蒸发等

3. **Reactions**
   - 规则判定（阈值/邻域/组合条件）→ 产出标准化 Command 列表

4. **Commit**
   - 唯一写回入口：执行 Commands，写入各域状态、flags、blocks、dirty、diff

5. **Post**
   - 清理活跃集合、打包 diff、客户端 remesh/vfx

> 领域系统只能注册到指定阶段，禁止随意跨阶段写入别的领域内部结构。

---

## 6. 跨域交互：Reaction/Command 模式

### 6.1 为什么要 Reaction/Commit
如果燃烧要读温度、写温度、改方块、加 flags、触发相变……域与域直接互相依赖会导致：
- 依赖环
- 修改牵一发动全身
- 系统顺序难以保证

### 6.2 标准化 Command（建议通用、可扩展）
设计一个小而稳定的命令集合：
- `SetBlock{idx, new_block}`
- `AddFlag{idx, flag}` / `RemoveFlag{...}`
- `AddHeat{idx, amount}`（由 thermal 域在 Commit 中执行）
- `SetTemp{idx, value}`（可选）
- `Ignite{idx, power}` / `Extinguish{idx, reason}`
- `SpawnAsh{idx, amount}`（或直接 SetBlock）

领域系统/反应系统只产出命令，不直接写别的域结构。

### 6.3 Commit 统一落地
- Commit 阶段按优先级/冲突规则处理同一 idx 的多条命令
- 统一写：
  - `blocks/flags/variant`
  - 各 domain state
  - `dirty_blocks`、`needs_remesh`
  - `changes(diff)`

---

## 7. 领域模块模板（新增一条“物理线”的标准做法）

> 每个领域模块遵循同一模板，保证可扩展。

一个 Domain 模块应包含：

1) **State**（存入 Chunk）
- `ChunkXxxState`：稀疏表/数组 + `active_xxx`

2) **API**（封装访问）
- `get_xxx(idx)` / `set_xxx(idx)` / `activate(idx)` / `deactivate(idx)`
- 任何写操作都要正确维护 active/diff/dirty（通常由 Commit 执行）

3) **Systems**（注册到指定 SystemSet）
- `xxx_field_update`（可选：如热扩散）
- `xxx_reaction_emit`（判定条件→产出 Commands）
- `xxx_tick`（消耗、衰减、传播等→产出 Commands）

4) **Defs/Params**（与 BlockDef 交互）
- 该领域相关参数尽量落在 `BlockDef` 或领域配置资源中

---

## 8. 以燃烧线为例：从点燃到长期变化的完整闭环

### 8.1 必备数据
- `BlockDef`：`is_flammable`、`ignition_temp`、`burn_energy`、`burn_rate`、`heat_release`、`char_block/ash_block`
- `flags[idx]`：`Burning`、`Charred`
- `burn_state: SparseMap<idx, BurnState>`
- `active_burning`
- `temp`（稀疏/按需）与 `active_thermal`

### 8.2 触发来源
- 外部点火：玩家/技能 → `Ignite{idx,power}`
- 温度阈值：`temp[idx] >= ignition_temp` 且可燃 → `Ignite`
- 相邻传播：燃烧 tick 对邻域注热/推温度过阈值

### 8.3 tick 行为（燃烧系统产出 Commands）
- 消耗燃料：`fuel_left -= burn_rate * intensity * dt`
- 放热：`AddHeat{idx, burned * heat_release}`
- 传播：对邻居 `AddHeat`（或 `Ignite`）
- 熄灭：燃料耗尽/被压制 → `Extinguish`

### 8.4 长期变化（Commit 阶段落地）
- `Extinguish` 后：
  - `SetBlock: Wood -> CharredWood -> Ash -> Air`（按阶段）
  - `RemoveFlag(Burning) / AddFlag(Charred)`
  - 标记 `needs_remesh`

### 8.5 客户端表现
- 客户端收到 diff：
  - `Burning flag` 驱动火焰/烟雾 VFX
  - `Charred` 驱动材质变黑
  - `needs_remesh` 触发局部重网格

---

## 9. 逐步完善路线图（建议迭代顺序）

> 目标：先跑通闭环，再逐步加“近现实”细节。

### Phase 1：最小可用闭环
- 仅支持外部点火 `Ignite`
- 燃烧 tick 消耗燃料
- 熄灭后 `SetBlock` 变焦黑块
- diff 同步 + 客户端 VFX

### Phase 2：温度场引入（物理化传播）
- 引入 `temp`（默认值+稀疏覆盖或按需分配）
- 燃烧向周围注热，热扩散系统更新 `active_thermal`
- 点燃规则由温度阈值触发（减少概率逻辑）

### Phase 3：湿度/抑制机制
- 引入 `moist` 与蒸发/湿润扩散
- 湿度提高点燃阈值或降低强度
- 加入玩家灭火、缺氧（可选）

### Phase 4：相变与多材料反应
- 水 ↔ 冰（温度阈值）
- 熔融/凝固、腐蚀等更多领域线接入 Reaction/Commit

### Phase 5：结构与塌落（高级）
- 引入强度/完整性衰减
- 崩塌生成碎屑或直接 SetBlock

### Phase 6：大规模 MMO 优化
- Chunk streaming（视距/区域加载卸载）
- 多线程/任务化（按 chunk 并行）
- 网络优先级（只同步玩家关注区）
- 存档压缩与 diff 合并

---

## 10. Bevy ECS 集成建议（必须但要克制）

### 10.1 用 ECS 的部分
- Chunk Entity 生命周期（spawn/despawn）
- FixedUpdate 调度与 SystemSet 顺序
- 网络收发与事件分发
- 客户端渲染（mesh/vfx）

### 10.2 尽量保持纯 Rust 的部分
- Chunk 内数据结构
- 各领域 tick 逻辑
- Reaction 规则评估
- Commit 冲突处理
- diff 生成与应用

> 这样你能：服务端无渲染复用同一套核心；核心逻辑可单测；未来迁移引擎/扩展也更稳。

---

## 11. 工程实践清单（落地时务必遵守）

- **禁止**：每个方块一个 Entity。
- **禁止**：每条领域线对全世界/全 chunk 全量扫描。
- **必须**：活跃集合驱动；状态按需分配/稀疏化。
- **必须**：系统按固定阶段执行；跨域通过命令提交。
- **必须**：所有可持久化变化写入 diff/dirty；渲染/网络从 diff 驱动。

---

## 12. 你下一步实现的最小骨架（建议顺序）

1) `Chunk`：`blocks + flags + dirty + changes`
2) `BlockDef` 注册表（至少：空气/木头/焦木/灰）
3) `CombustionDomain`：`burn_state + active_burning`
4) `Reaction`（先硬编码：IgniteRequest → Burning）
5) `Commit`：执行 Ignite/Extinguish/SetBlock/AddFlag
6) `Client Vfx`：Burning flag → 火焰粒子
7) `diff`：服务端→客户端同步 blocks/flags 变化
8) 再引入 thermal/moisture/phase_change

---

## 13. 术语对照（便于团队沟通）

- **Block/Voxel**：格子单元，不是 Entity
- **Chunk**：固定尺寸体素容器，是 Entity
- **Domain**：一条物理/属性线（热/湿/燃烧/相变…）
- **Reaction**：跨域规则（条件→效果）
- **Commit**：唯一写回入口（落地命令、写 diff/dirty）
- **Diff**：增量变化记录（网络/存档）

---

> 本文档作为“纲领性参考”。后续新增领域线时，严格遵循：**Domain 模板 + Reaction/Commit 管线 + 活跃集合驱动**。这样你可以持续扩展复杂度，而不牺牲性能与可维护性。

