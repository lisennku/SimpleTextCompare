# SimpleTextCompare — Replace/行内高亮 改造交接摘要

> 用途：跨对话快速对接。新会话开头贴这份即可继续。

## 0. 协作约定
- 称呼用户：**聪叔**
- 模式：**只引导、指方向、提问**，不直接给答案或完整代码（除非明确要求）
- 查资料：**联网读 docs.rs**，不读本机文件

## 1. 项目 & 目标
- 项目：`C:\Users\LFJ\RustroverProjects\SimpleTextCompare`，Rust CLI，左右对照表格 + ANSI 颜色
- 命令：`stc diff left right [-p dir] [--less]`
- **现状**：`compare.rs` 用 `diff.iter_all_changes()` 逐行输出，改动行被拆成上下两行（Delete + Insert）
- **目标**：改动行合并成**一行**，状态标 **Replace**，行内只把**变化的片段**（如 `1.0.0`→`1.1.0` 里的 `0`→`1`）标色（黄色等），左右并排对照

## 2. 已确认的技术结论（similar 3.2.0，均已本地实测）

### TextDiff 内部存储
- 6 个字段：`old`/`new` = `TextDiffSide<'a>`，`ops` = `Vec<DiffOp>`，`newline`/`whitespace_mode`/`algorithm`（配置/标志位，非内容）
- `TextDiffSide` 是 `pub(crate)` enum：
  - `BorrowedTokens(Vec<&'a T>)`
  - `OwnedTokens(Vec<T::Owned>)`
- `from_lines(&s1,&s2)` 走 **Borrowed**：零拷贝，token 是借 s1/s2 的 `&str`，且**保留行尾 `\n`**

### DiffOp vs Change vs InlineChange
- `DiffOp` = 一个"**块**"，只有范围（`old_range()`/`new_range()` → `Range<usize>`），**没有 value**
- `Change` = 单个元素，**有 `value()`**（from_lines 时是 `&str`）
- 取某 op 的行文本：`for i in op.old_range()` → `diff.old_slice(i)` → `Option<&str>`（收**单个** index）→ 去 Option → `collect::<Vec<&str>>()`；new 侧对称用 `new_range()`/`new_slice()`

### iter_inline_changes（已实测）
- 签名：**`iter_inline_changes(op)` 收单个 `&DiffOp`**（docs 摘要说的 `&[DiffOp]` 是错的，实测传单个 op 可编译）
- 返回 `Iterator<Item = InlineChange>`；`InlineChange` 字段：**`tag` / `old_index` / `new_index` / `values`**
- `values()` = **`Vec<(bool, &str)>`**，`bool=true` 表示该片段是行内变化、需 emphasis（上色）
- **`iter_all_inline_changes()` vs `iter_inline_changes(op)`**：前者内部帮你把所有 op 遍历并拼接（不传参）；后者只处理传入的单个 op。二者是**二选一入口，不可嵌套**（把 all 版放进 `for op` 循环会把整个 diff 重复打印 N 遍）
- **决定用非 all 版**：因为它**保留了 op 的块边界**，做左右配对需要这个边界

### 分支模型（已达成共识）
- **权威信号是 `op.tag()`**（Equal/Delete/Insert/Replace），**不要靠数块内有几条 Delete/Insert 来反推**（多行 Replace 会打破"1 Delete + 1 Insert"）
- **一个 op ≠ 一行**：op 是块，可跨多行（尤其 Equal，一长串相同行会打包成一个 op，`old_len` 可能几十）→ 必须在 op 内部再 loop 每一行
- Replace 块内：`change.tag()==Delete` = **左/旧**侧（带 old_index），`==Insert` = **右/新**侧（带 new_index），各自 `values()` 标好行内哪些字符变了
- **Equal/Delete/Insert 用不到 `iter_inline_changes`**：不需要行内高亮，直接 `old_range`/`new_range` + `old_slice`/`new_slice` 取纯文本行即可。`iter_inline_changes` **只有 Replace 才用**
- **行号**：改用 `op.old_range().start + 1` / `new_range().start + 1` 推导，弃用手动 `+= 1`

| tag | old 侧 | new 侧 | 行内高亮 | 用哪个函数 |
|---|---|---|---|---|
| Equal | 取 | 取 | 否 | 复用老 output |
| Delete | 取 | 不取（空） | 否 | 复用老 output |
| Insert | 不取（空） | 取 | 否 | 复用老 output |
| Replace | 取 | 取 | **是（唯一需要）** | 新函数（inline 开时） |

## 3. 核心难点：ANSI 与宽度计算
- **铁律**：永远不要拿"已含 ANSI 的字符串"去算宽度。`\x1b[33m` 里 `ESC` 算 0 宽，但 `[`/`3`/`3`/`m` 各算 1 宽 → 一个颜色码被误当成 **4 个可见宽度**，列宽算歪、对齐全乱
- 现有 `output.rs::padding_white_space` 已经踩对了点：它**先量纯文本宽度、再 `wrap_ansi` 上色**，所以整格染色不翻车
- **Replace 逐片段染色的难点**：wrap 和染色**必须同步做**
  - 死路 A：先上色再 wrap → wrap 按含 ANSI 长度算 → 歪
  - 死路 B：先 wrap 纯文本再上色 → wrap 后丢了"哪几个字符是 emphasis"的信息（values 片段边界被 wrap 切断）
  - 出路：布局单位从"一个 `&str`"升级为"values 那串 `(bool,&str)` 片段"，一边按**可见宽度**塞字符/决定换行，一边记住是否 emphasis，**输出那一刻**才给 emphasis 片段套 ANSI
- **更细的坑**：一段 emphasis 文字**跨换行**被切到两个物理行时，每个物理行是单独 `writeln!` 出去的，ANSI 状态不会自动延续 → 续行开头要**重新开启颜色**（末尾照常 RESET）

## 4. 架构决定：路线 B（统一 ops 一条路）
- **选定路线 B**：主循环直接换成 `ops()` 版，所有 tag 都走它；**只有 Replace 那一步用 inline flag 决定**：
  - flag 开 → 走新的逐片段染色（合并成一行）
  - flag 关（默认）→ 退回老效果：Replace 的 old 行按 Delete、new 行按 Insert 打两行（= 现在 `iter_all_changes` 的输出）
- 前提：ops 版渲染 Equal/Delete/Insert 的输出**必须和现在逐字节相同**（靠 baseline 验证）

### 改动范围账本（不是推倒重来）
- **compare.rs**：只有 `compare_files_table_style` 的**循环体**动（`iter_all_changes` → `ops()` + match 四 tag）；读文件/建 diff/取文件名/打 header+separator **全不动**
- **output.rs**：**纯新增**——加一个 ANSI-aware、逐片段染色的 Replace 专用函数；现有 `wrap_code_width`/`padding_white_space`/`output_wrapped_row`/header/separator **一行不改**，Equal/Delete/Insert 继续用
- **line_status.rs**：enum 加 `Replace` 变体 + 一个颜色（黄）。小改
- **cli.rs / app.rs**：`Compare` 加 `inline` 字段，app 取出传进 `compare_files_table_style`。小改

## 5. 执行计划（分步、每步独立可验证）

### 第 0 步：存 baseline（动代码之前！）
用**没改过的代码**跑默认输出存文件：
```
cargo run -- diff left.txt right.txt -p <dir> > baseline.txt
```
不加 `--less`（走 stdout，ANSI 会进文件，两边一致即可）。这是判分标准答案。

### 第 1 步：只做 ops() 迁移，先不碰 inline
主循环换成 `ops()` + match 四 tag，**这版 Replace 仍走老样子**（old 行 Delete、new 行 Insert 打两行）。目标：**输出与 baseline 逐字节相同**。
- Equal：遍历 `old_range`，每行 → `output_wrapped_row(左号,行,右号,行,Equal)`
- Delete：遍历 `old_range` → `(左号,行,None,None,Delete)`
- Insert：遍历 `new_range` → `(None,None,右号,行,Insert)`
- Replace（默认）：old 行全按 Delete、new 行全按 Insert
- 行号用 `range.start+1`
- 跑 `diff baseline.txt 新输出`，**必须全等** = 证明换 ops() 没改坏东西
- ⚠️ **别猜** Replace 默认版 Delete/Insert 的先后顺序，**让 baseline 的 diff 告诉你**，对不上照 diff 调

### 第 2 步：baseline 绿了之后，才加 inline
- 加 flag（cli.rs + app.rs 串线）
- `line_status.rs` 加 `Replace` 变体 + 黄色
- output.rs 新增 Replace 专用的 ANSI-aware wrap + 逐片段染色函数
- compare.rs 的 Replace 分支：flag 开 → 调新函数
- 因为第 1 步已消掉"重构有没有改坏"这个变量，第 2 步输出若不对，**立刻能定位是 inline 新功能的锅**

> 为啥分步：一次同时干两件事（换 ops + 加 inline），输出错了分不清谁引起的。分两步 = 每步只动一个变量。

## 6. 当前进度
- 实验文件：`examples/similar_inline_change_demo.rs`，已切到真实文件（`C:\Users\LFJ\Desktop\Compare` 的 left.txt / right.txt）
- 已跑通：`ops()` + `op.tag()` 打印、`old_range()` + `old_slice(i)` 取行、`iter_inline_changes(op)` 打印出 `InlineChange`（tag/old_index/new_index/values 结构已确认）
- 尚未开始改 `src/` 下任何代码；尚未存 baseline

## 7. 未决问题
1. **多行 Replace 的左右配对策略**：`old_len ≠ new_len`（如左 2 行、右 3 行）时怎么配成"一行两列"？1:1 好办，不等的待定。**先做通 1:1，再收拾这个边界**
2. **inline 开关做成 CLI flag（`--inline`）还是配置项（写进 `stc.toml`）**？未定（config 那套已支持配置项，两条路都通）
3. Replace 逐片段渲染的数据结构设计（片段 + emphasis + 颜色 + 跨行续色）

## 8. 关键文件
- `src/compare.rs`（`compare_files_table_style`，核心循环）
- `src/line_status.rs`（`LineStatus` enum + `wrap_ansi`）
- `src/output.rs`（`wrap_code_width` / `padding_white_space` / `output_wrapped_row` / header / separator）
- `src/ansi_config.rs`（颜色常量，`YELLOW` 已有）
- 外围：`app.rs` / `cli.rs`（加 flag 时动）；`config.rs` / `pagers.rs`（暂不动）
