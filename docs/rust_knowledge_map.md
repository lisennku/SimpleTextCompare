# SimpleTextCompare 涉及的 Rust 知识点地图

> 本文档按**主题分类**整理了 `SimpleTextCompare`(stc)整个代码库涉及的 Rust 知识点。
> 每条格式:`概念 — 一句话说明 · 代码位置 · 文档链接`。
> 第三方 crate 的用法单列在第 13 节;全部链接的去重汇总在第 14 节(速查表)。
>
> 文档来源:
> - **The Rust Book**(官方书,有中文版)—— 概念主力
> - **Rust By Example** —— 看例子最快
> - **std API 文档**(doc.rust-lang.org/std)—— 具体类型/方法
> - **Rust Reference** —— 语法精确规范
> - **docs.rs** —— 第三方 crate
>
> 建议先啃(出现频率最高):`&[T]` 切片(第 7 节)、`iter().map().collect()` + 闭包(第 5 节)、Option/Result(第 3 节)、`match`/`let-else`(第 4 节)、所有权借用(第 1 节)。

---

## 1. 所有权 · 借用 · 生命周期

- **所有权与移动** — `finish(self)` 按值接收,消费掉 Pager;`rows.extend(owned_vec)` 把整个 Vec 移进去 · `pagers.rs:61`、`row.rs:210` · https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
- **不可变借用 `&T`** — `build_rows(&diff, ...)`、`render_rows(&rows, ...)` 只借不拥有 · `compare.rs:65-66` · https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
- **可变借用 `&mut T`** — `writer: &mut dyn Write`、`get_mut(i)` 拿 `&mut` · `output.rs:169`、`row.rs:177` · 同上
- **不能从借用里移出值** — 所以用 `get_mut(i)` + `no.take()` 而非 `*line` · `row.rs:176-184` · https://doc.rust-lang.org/std/option/enum.Option.html#method.take
- **`self` 的三种形态** — `&self`(借)、`&mut self`(mut 借)、`self`(消费) · `line_status.rs:20`、`pagers.rs:50`、`pagers.rs:61` · https://doc.rust-lang.org/book/ch05-03-method-syntax.html
- **生命周期省略** — `piece_color(&self,...) -> Option<&str>` 返回的 `&str` 生命周期被自动绑到 `&self` · `line_status.rs:41` · https://doc.rust-lang.org/reference/lifetime-elision.html

## 2. struct · enum · 元组

- **struct 定义 + 字段简写初始化** — `Row { left_no, right_no, ... }`(变量名=字段名时可简写) · `row.rs:44-50`、`row.rs:60-66` · https://doc.rust-lang.org/book/ch05-01-defining-structs.html
- **关联函数(构造函数)** — `Row::new(...)`、`Pager::new(...)`,用 `::` 调,不带 self · `row.rs:53`、`pagers.rs:29` · https://doc.rust-lang.org/book/ch05-03-method-syntax.html
- **enum 三种变体** — 单元变体 `Equal`、元组变体 `Stdout(io::Stdout)`、结构体变体 `Less { child, writer }` · `line_status.rs:12-17`、`pagers.rs:22-25` · https://doc.rust-lang.org/rust-by-example/custom_types/enum.html
- **元组 + 解构** — `(bool, String)`、`let (left_no, left_line) = ...`、`.1` 按索引取 · `output.rs:118`、`row.rs:177`、`output.rs:132` · https://doc.rust-lang.org/book/ch03-02-data-types.html
- **嵌套泛型类型** — `Option<Vec<(bool, String)>>`、`Vec<(usize, Vec<(bool,String)>)>` · `row.rs:47`、`output.rs:118` · https://doc.rust-lang.org/book/ch10-01-syntax.html

## 3. Option 与 Result(全代码的主力)

- **`Option<T>`** — 表达"可能没有":`Option<usize>` 行号、`Option<PathBuf>` · `row.rs:45`、`cli.rs:101` · https://doc.rust-lang.org/std/option/enum.Option.html
- **`Option::map`** — 有值就变换:`old_index().map(|i| i+1)`、`old_slice(o).map(plain_seg)`(直接传**函数名**当参数) · `row.rs:141`、`row.rs:86` · https://doc.rust-lang.org/std/option/enum.Option.html#method.map
- **`Option::as_deref`** — `Option<Vec<T>>` → `Option<&[T]>`,一步借成切片 · `output.rs:173` · https://doc.rust-lang.org/std/option/enum.Option.html#method.as_deref
- **`and_then` / `unwrap_or` / `unwrap_or_else`** — 链式取文件名 `file_name().and_then(...).unwrap_or("<left>")`;惰性默认 `unwrap_or_else(|| ...)` · `compare.rs:42-45`、`pagers.rs:34` · https://doc.rust-lang.org/std/option/enum.Option.html
- **`ok_or` / `ok_or_else`** — 把 Option 转成 Result(None→Err),好接 `?` · `config.rs:163`、`pagers.rs:44` · https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else
- **`Result<T, E>`** — 可恢复错误 · `config.rs:47`、`compare.rs:35` · https://doc.rust-lang.org/std/result/enum.Result.html
- **`Result::map_err`** — 换错误类型:`parse().map_err(|_| anyhow!(...))` · `cli.rs:41` · https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err

## 4. 模式匹配

- **`match` 穷尽匹配** — 对 enum 每个变体给分支 · `row.rs:81`、`line_status.rs:33` · https://doc.rust-lang.org/book/ch06-02-match.html
- **匹配元组 + 嵌套模式** — `match (index, line_no) { (0, Some(n)) => ... }`(字面量 0 套 Some) · `output.rs:33-35` · https://doc.rust-lang.org/reference/patterns.html
- **通配 `_` 与捕获兜底** — `_ => "/"`、`tag => {...}`(把没匹配的值绑给 tag)、`_ => unreachable!()` · `output.rs:35`、`row.rs:99`、`row.rs:170` · https://doc.rust-lang.org/book/ch06-02-match.html
- **`if let` / `while let`** — 只关心一个分支:`if let Some(x) = ...`、`if let Err(err) = res` · `app.rs:27`、`app.rs:76` · https://doc.rust-lang.org/book/ch06-03-if-let.html
- **`let ... else`(let-else)** — 解构失败就走 else 提前返回:`let Some(segs) = segs else { return ... }` · `output.rs:114`、`cli.rs:62` · https://doc.rust-lang.org/reference/statements.html
- **match ergonomics(自动解引用绑定)** — `for (is_emphasis, line) in segs` 对 `&[(bool,String)]`,自动绑成 `&bool`/`&String`;`Some((no,line))` 对 `&mut (...)` 自动绑 `&mut` · `output.rs:122`、`row.rs:178` · https://doc.rust-lang.org/reference/patterns.html
- **`..` 剩余模式** — `Pager::Less { writer, .. }` 忽略其余字段 · `pagers.rs:53` · https://doc.rust-lang.org/reference/patterns.html

## 5. 迭代器与闭包

- **闭包 `|args| body`** — 匿名函数,当参数传给 map 等:`|i| i+1`、`|(b,s)| (*b, s...to_string())`、带块 `|(w, codes)| { ... }` · `row.rs:141`、`row.rs:146`、`output.rs:150` · https://doc.rust-lang.org/rust-by-example/fn/closures.html
- **`iter()` vs `into_iter()`** — `iter()` 借(出 `&T`)、`into_iter()` 消费(出 owned `T`) · `row.rs:145`、`output.rs:149` · https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
- **`map().collect()`** — 逐元素变换再收集成容器(目标类型靠上下文推断) · `row.rs:145-152`、`output.rs:148-162` · https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect
- **`zip`** — 两个迭代器并行走:`old_range().zip(new_range())` · `row.rs:83` · https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.zip
- **`Range` 迭代** — `for i in 0..max_lines_cnt`、`for o in op.old_range()` · `output.rs:188`、`row.rs:101` · https://doc.rust-lang.org/std/ops/struct.Range.html
- **`chars()`** — 按 Unicode 标量逐字符遍历 · `output.rs:124` · https://doc.rust-lang.org/std/primitive.str.html#method.chars

## 6. 字符串:String / &str / char

- **`String`(拥有) vs `&str`(借用视图)** — `to_string()` 把 &str 升成 String;`&String` 能自动降级成 `&str`(deref) · `row.rs:39`、`output.rs:159` · https://doc.rust-lang.org/book/ch08-02-strings.html
- **`str` 常用方法** — `trim_end_matches('\n')`、`repeat(n)`、`parse::<usize>()` · `row.rs:37`、`output.rs:62`、`cli.rs:41` · https://doc.rust-lang.org/std/primitive.str.html
- **`push_str` / `push`** — 往 String 追加串/字符 · `output.rs:154`、`output.rs:132` · https://doc.rust-lang.org/std/string/struct.String.html
- **`char` 字面量与 `to_string`** — `'\n'`、`'\r'`;`ch.to_string()` · `row.rs:37`、`output.rs:134` · https://doc.rust-lang.org/std/primitive.char.html
- **原始字符串 `r"..."`** — 不转义,专治 Windows 路径 `r"C:\Users\..."` · `config.rs:10`、`row.rs:226` · https://doc.rust-lang.org/reference/tokens.html
- **转义序列 `\x1b`** — ANSI 颜色码里的 ESC · `ansi_config.rs:3` · https://doc.rust-lang.org/reference/tokens.html

## 7. Vec 与切片 `&[T]`

- **`Vec<T>`** — 可增长数组;`Vec::new()`、`push`、`len` · `row.rs:80`、`output.rs:119` · https://doc.rust-lang.org/book/ch08-01-vectors.html
- **`vec!` 宏** — 快速建 Vec:`vec![(false, ...)]` · `row.rs:35` · https://doc.rust-lang.org/std/macro.vec.html
- **切片 `&[T]`** — 一段连续数据的只读视图,比 `&Vec<T>` 更通用(`&Vec` 能自动转 `&[T]`) · `output.rs:109`、`output.rs:166` · https://doc.rust-lang.org/std/primitive.slice.html
- **`get` / `get_mut`** — 越界安全的索引,返回 `Option<&T>` / `Option<&mut T>` · `output.rs:192`、`row.rs:177` · https://doc.rust-lang.org/std/primitive.slice.html#method.get
- **`last` / `last_mut`** — 取末元素(Option) · `output.rs:131-132` · https://doc.rust-lang.org/std/primitive.slice.html#method.last
- **`extend`** — 把另一个可迭代对象的元素并入(消费它) · `row.rs:210` · https://doc.rust-lang.org/std/vec/struct.Vec.html#method.extend
- **`is_empty`** — 判空(比 `len()==0` 惯用) · `output.rs:140`、`output.rs:144` · https://doc.rust-lang.org/std/vec/struct.Vec.html#method.is_empty

## 8. 错误处理:? 与 anyhow

- **`?` 运算符** — Err 就提前 return,Ok 就解包继续;贯穿全代码 · `app.rs:13`、`compare.rs:36`、`output.rs:89` · https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
- **`panic!` / `unreachable!`** — 不可恢复;`unreachable!()` 标记"逻辑上到不了"的分支 · `row.rs:170`、`line_status.rs:37` · https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html
- **`anyhow::Result` / `bail!` / `anyhow!`** — 应用级错误:统一 Result、造错误、提前返回错误 · `cli.rs:35`、`cli.rs:44`、`pagers.rs:44` · https://docs.rs/anyhow
- **`Context::with_context`** — 给错误加上下文说明 · `compare.rs:36`、`config.rs:113` · https://docs.rs/anyhow/latest/anyhow/trait.Context.html
- **`downcast_ref` 错误降级** — 把 anyhow::Error 试着还原成具体类型(io::Error)判断 BrokenPipe · `app.rs:78-81` · https://docs.rs/anyhow/latest/anyhow/struct.Error.html#method.downcast_ref
- **`From`/`Into` 错误转换** — `err.into()` · `app.rs:83` · https://doc.rust-lang.org/std/convert/trait.From.html

## 9. trait:derive · impl · dyn · 泛型

- **`impl Trait for Type`** — 给类型实现 trait:`impl Default for AppConfig` · `config.rs:67` · https://doc.rust-lang.org/rust-by-example/trait.html
- **inherent impl(固有实现)** — `impl LineStatus {...}`、`impl cli::Cli {...}`(给自己/别处的类型加方法) · `line_status.rs:19`、`app.rs:7` · https://doc.rust-lang.org/book/ch05-03-method-syntax.html
- **`derive` 派生宏** — `#[derive(Debug, Copy, Clone)]`、`#[derive(Serialize, Deserialize)]` 自动生成 trait 实现 · `line_status.rs:10`、`config.rs:24` · https://doc.rust-lang.org/reference/attributes/derive.html
- **`Copy` / `Clone`** — Copy 让 LineStatus 能按位复制(所以能到处传值) · `line_status.rs:10` · https://doc.rust-lang.org/std/marker/trait.Copy.html
- **`Debug` 与 `{:?}`** — 派生后能用 `println!("{:?}", x)` 打印 · `row.rs:43`、`row.rs:236` · https://doc.rust-lang.org/std/fmt/trait.Debug.html
- **trait object `dyn Write`** — "任何实现了 Write 的东西",运行时多态;`&mut dyn Write` 让 stdout/子进程管道都能塞 · `output.rs:169`、`pagers.rs:50` · https://doc.rust-lang.org/reference/types/trait-object.html
- **泛型参数** — `TextDiff<str>` 给类型传参 · `row.rs:79` · https://doc.rust-lang.org/book/ch10-01-syntax.html
- **turbofish `::<T>`** — 调用处显式指定类型:`parse::<usize>()`、`downcast_ref::<io::Error>()` · `cli.rs:41`、`app.rs:79` · https://doc.rust-lang.org/book/ch10-01-syntax.html

## 10. 宏与属性

- **`format!` / `writeln!` / `println!`** — 格式化;`writeln!` 写进 writer 并返回 io::Result · `output.rs:64`、`output.rs:89`、`output.rs:196` · https://doc.rust-lang.org/std/macro.writeln.html
- **格式化语法** — `{}` 位置参数、`{name}` 直接捕获变量、`{:<20}` 左对齐宽度 · `output.rs:184`、`ansi_config.rs:41` · https://doc.rust-lang.org/std/fmt/index.html
- **`matches!`** — 判断值是否匹配某模式,返回 bool(省去写 match):`matches!(tag, Delete | Replace)` · `row.rs:100`、`row.rs:209` · https://doc.rust-lang.org/std/macro.matches.html
- **`|` or-pattern** — 一个分支匹配多个:`Delete | Replace` · `row.rs:100` · https://doc.rust-lang.org/reference/patterns.html
- **outer attribute `#[...]`** — 作用于下一项:`#[allow(dead_code)]`、`#[test]`、`#[arg(...)]` · `output.rs:24`、`row.rs:224`、`cli.rs:100` · https://doc.rust-lang.org/reference/attributes.html
- **inner attribute `#![...]`** — 作用于整个模块/crate:`#![allow(dead_code)]` · `ansi_config.rs:1` · https://doc.rust-lang.org/reference/attributes.html
- **`#[cfg(test)]` 条件编译** — 测试模块只在 test 时编译 · `row.rs:219`、`output.rs:211` · https://doc.rust-lang.org/book/ch11-01-writing-tests.html

## 11. 模块系统与可见性

- **`mod` 声明模块** — main.rs 里 `mod cli; mod app; ...` 把各文件纳入 crate · `main.rs:3-11` · https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html
- **`use` 引入路径** — `use crate::row::Row;`、`use std::io::{self, Write};`(`self` 把 io 本身也引进来) · `output.rs:18-22`、`app.rs:5` · https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html
- **`crate::` 绝对路径** — 从 crate 根开始定位 · `output.rs:18` · https://doc.rust-lang.org/reference/paths.html
- **`pub` 可见性** — 不加 pub 只能本模块用(format_side 是 pub,assemble_* 不是) · `row.rs:34`、`row.rs:79` · https://doc.rust-lang.org/reference/visibility-and-privacy.html
- **`use super::*`** — 测试模块引入父模块全部内容 · `row.rs:221` · https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html
- **`const` 常量** — 编译期常量,全大写命名 · `output.rs:28`、`ansi_config.rs:3` · https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html

## 12. 标准库:IO · 文件 · 进程

- **`io::Write` trait** — `write!`/`writeln!`/`flush` 的来源 · `output.rs:21`、`pagers.rs:64` · https://doc.rust-lang.org/std/io/trait.Write.html
- **`io::Result` / `io::Error` / `ErrorKind`** — IO 专用结果与错误种类(BrokenPipe) · `output.rs:84`、`app.rs:80` · https://doc.rust-lang.org/std/io/type.Result.html
- **`IsTerminal`** — 判断是否终端,决定开不开颜色 · `app.rs:64`、`pagers.rs:30` · https://doc.rust-lang.org/std/io/trait.IsTerminal.html
- **`fs::read_to_string` / `fs::write` / `create_dir_all`** — 文件读写、建目录 · `compare.rs:36`、`config.rs:120` · https://doc.rust-lang.org/std/fs/fn.read_to_string.html
- **`Path` / `PathBuf`** — 路径:`join`、`file_name`、`display`、`try_exists`、`as_os_str` · `compare.rs:42`、`config.rs:114` · https://doc.rust-lang.org/std/path/struct.PathBuf.html
- **`OsStr::to_str`** — 把系统字符串转 `&str`(可能失败,返回 Option) · `compare.rs:44` · https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.to_str
- **`env::home_dir`** — 取用户主目录 · `config.rs:160` · https://doc.rust-lang.org/std/env/fn.home_dir.html
- **`process::Command` 起子进程** — builder 式:`Command::new(..).arg(..).stdin(..).spawn()`;`Child`/`ChildStdin`/`Stdio::piped` · `pagers.rs:36-39` · https://doc.rust-lang.org/std/process/struct.Command.html
- **`drop()` 显式释放** — 关掉 less 的 stdin 好让它结束 · `pagers.rs:72` · https://doc.rust-lang.org/std/mem/fn.drop.html
- **`usize` 数值方法** — `saturating_sub`(减到 0 不下溢)、`max` · `output.rs:158`、`output.rs:176` · https://doc.rust-lang.org/std/primitive.usize.html

## 13. 第三方 crate

- **clap(命令行解析)** — `#[derive(Parser/Subcommand/Args)]` 用结构体声明 CLI;`#[arg(...)]`/`#[command(...)]` 配属性;`Cli::parse()` 解析 · `cli.rs:36`、`cli.rs:77-140`、`main.rs:14` · https://docs.rs/clap
  - **`#[arg(...)]` 为啥在 API 索引里搜不到** — 它是过程宏属性、不是普通 item(struct/fn/trait),所以不进 docs.rs 左侧索引;clap 把 derive 属性单放进 `_derive`("Derive Reference")文档模块,开头的下划线让它排序怪、极易被忽略 · 进去看 "Attributes → Arg" 节 · https://docs.rs/clap/latest/clap/_derive/index.html
  - **`#[arg(...)]` 专属魔法属性** — `id`、`value_parser`、`action`、`help`、`long_help`、`verbatim_doc_comment`、`short`、`long`、`env`、`from_global`、`value_enum`、`skip`、`default_value`、`default_value_t`、`default_values_t`、`default_value_os_t`、`default_values_os_t`
  - **关键:`clap::Arg` 的任意方法都能当属性用** — 官方原文 "Any `Arg` method can also be used as an attribute";所以"#[arg()] 能填什么"的全量 = 上面魔法属性 + `Arg` 方法列表;`.required()`→`#[arg(required = true)]`、`.num_args()`→`#[arg(num_args = 1..)]`、`.value_name()`→`#[arg(value_name = "FILE")]` · https://docs.rs/clap/latest/clap/struct.Arg.html
  - **属性语法映射** — 方法收 bool/无参 → 光杆 flag(`.short()`→`#[arg(short)]`);方法收值 → `= 值`(`.default_value("x")`→`#[arg(default_value = "x")]`);多值逗号并列(`#[arg(num_args = 1, value_name = "F")]`)
  - **配套页 & 版本对齐** — derive 教程 https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html;cookbook(按场景查写法)https://docs.rs/clap/latest/clap/_derive/_cookbook/index.html;把 URL 里 `latest` 换成本项目版本 `4.6.6` 即对齐
- **serde(序列化框架)** — `#[derive(Serialize, Deserialize)]` 让 AppConfig 能和 TOML 互转 · `config.rs:4`、`config.rs:24` · https://docs.rs/serde
- **toml(TOML 编解码)** — `to_string_pretty`(存)、`from_str`(读) · `config.rs:113`、`config.rs:127` · https://docs.rs/toml
- **similar(diff 算法)** — `TextDiff::from_lines` 建差异;`diff.ops()` 拿操作块;`DiffOp` 的 `tag/old_range/new_range`;`old_slice/new_slice` 取文本;`iter_inline_changes` 做行内差异;`DiffTag`(Equal/Delete/Insert/Replace)、`ChangeTag`、`InlineChange` 的 `values()`(返回 `&[(bool,&str)]`,bool 即是否强调) · `compare.rs:40`、`row.rs:32`、`row.rs:83-111`、`row.rs:138-168` · https://docs.rs/similar
- **anyhow(错误处理)** — 见第 8 节 · `cli.rs:35` · https://docs.rs/anyhow
- **unicode-width(显示宽度)** — `UnicodeWidthStr::width(&str)`、`UnicodeWidthChar::width(char)`,算中日韩全角字符占几列——折叠/补齐的核心 · `output.rs:22`、`output.rs:52`、`output.rs:125` · https://docs.rs/unicode-width

---

## 14. 全部文档链接汇总(按来源去重 · 速查表)

### A. The Rust Book —— 系统教程(按章排好,可直接当学习路径)
入口:https://doc.rust-lang.org/book/ (中文版搜 "Rust 程序设计语言 中文版")
- 第3章 数据类型(元组)/ 常量 — https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html · https://doc.rust-lang.org/book/ch03-02-data-types.html
- 第4章 所有权 — https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
- 第4章 引用与借用 — https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
- 第5章 定义 struct — https://doc.rust-lang.org/book/ch05-01-defining-structs.html
- 第5章 方法与 self — https://doc.rust-lang.org/book/ch05-03-method-syntax.html
- 第6章 match — https://doc.rust-lang.org/book/ch06-02-match.html
- 第6章 if let — https://doc.rust-lang.org/book/ch06-03-if-let.html
- 第7章 模块/路径/use — https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html · https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html · https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html
- 第8章 Vec — https://doc.rust-lang.org/book/ch08-01-vectors.html
- 第8章 String/&str — https://doc.rust-lang.org/book/ch08-02-strings.html
- 第9章 panic 不可恢复错误 — https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html
- 第9章 Result 与 `?` — https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
- 第10章 泛型/turbofish — https://doc.rust-lang.org/book/ch10-01-syntax.html
- 第11章 写测试(`#[cfg(test)]`) — https://doc.rust-lang.org/book/ch11-01-writing-tests.html

### B. Rust By Example —— 看例子最快
入口:https://doc.rust-lang.org/rust-by-example/
- enum 三种变体 — https://doc.rust-lang.org/rust-by-example/custom_types/enum.html
- 闭包 — https://doc.rust-lang.org/rust-by-example/fn/closures.html
- trait 与 impl — https://doc.rust-lang.org/rust-by-example/trait.html

### C. std API 文档 —— 查具体类型/方法
入口:https://doc.rust-lang.org/std/
- **Option / Result**:Option(map/as_deref/take/and_then/unwrap_or/ok_or_else 各锚点)— https://doc.rust-lang.org/std/option/enum.Option.html ;Result(map_err)— https://doc.rust-lang.org/std/result/enum.Result.html
- **迭代器**:Iterator(map/collect/zip)— https://doc.rust-lang.org/std/iter/trait.Iterator.html ;IntoIterator(iter/into_iter)— https://doc.rust-lang.org/std/iter/trait.IntoIterator.html ;Range — https://doc.rust-lang.org/std/ops/struct.Range.html
- **字符串**:str(chars/parse)— https://doc.rust-lang.org/std/primitive.str.html ;String — https://doc.rust-lang.org/std/string/struct.String.html ;char — https://doc.rust-lang.org/std/primitive.char.html
- **集合**:slice(get/get_mut/last)— https://doc.rust-lang.org/std/primitive.slice.html ;Vec(extend/is_empty)— https://doc.rust-lang.org/std/vec/struct.Vec.html ;usize(saturating_sub/max)— https://doc.rust-lang.org/std/primitive.usize.html
- **trait / 转换**:Copy — https://doc.rust-lang.org/std/marker/trait.Copy.html ;Debug — https://doc.rust-lang.org/std/fmt/trait.Debug.html ;Default — https://doc.rust-lang.org/std/default/trait.Default.html ;From/Into — https://doc.rust-lang.org/std/convert/trait.From.html ;Deref — https://doc.rust-lang.org/std/ops/trait.Deref.html
- **宏 / 格式化**:writeln! — https://doc.rust-lang.org/std/macro.writeln.html ;format! 语法总览 — https://doc.rust-lang.org/std/fmt/index.html ;vec! — https://doc.rust-lang.org/std/macro.vec.html ;matches! — https://doc.rust-lang.org/std/macro.matches.html
- **IO / 文件 / 进程**:io::Write — https://doc.rust-lang.org/std/io/trait.Write.html ;io::Result — https://doc.rust-lang.org/std/io/type.Result.html ;IsTerminal — https://doc.rust-lang.org/std/io/trait.IsTerminal.html ;fs::read_to_string — https://doc.rust-lang.org/std/fs/fn.read_to_string.html ;PathBuf/Path — https://doc.rust-lang.org/std/path/struct.PathBuf.html ;OsStr::to_str — https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.to_str ;env::home_dir — https://doc.rust-lang.org/std/env/fn.home_dir.html ;process::Command — https://doc.rust-lang.org/std/process/struct.Command.html ;mem::drop — https://doc.rust-lang.org/std/mem/fn.drop.html

### D. Rust Reference —— 语法精确规范(抠细节时看)
入口:https://doc.rust-lang.org/reference/
- 模式(match ergonomics / or-pattern / `..` / 元组嵌套)— https://doc.rust-lang.org/reference/patterns.html
- 语句(let-else)— https://doc.rust-lang.org/reference/statements.html
- 生命周期省略 — https://doc.rust-lang.org/reference/lifetime-elision.html
- trait object(`dyn`)— https://doc.rust-lang.org/reference/types/trait-object.html
- 属性(outer `#[...]` / inner `#![...]` / derive)— https://doc.rust-lang.org/reference/attributes.html · https://doc.rust-lang.org/reference/attributes/derive.html
- 词法:原始字符串 / 转义 — https://doc.rust-lang.org/reference/tokens.html
- 路径与可见性 — https://doc.rust-lang.org/reference/paths.html · https://doc.rust-lang.org/reference/visibility-and-privacy.html

### E. docs.rs —— 第三方 crate
- clap(命令行;Derive 参考/`#[arg]` 属性 https://docs.rs/clap/latest/clap/_derive/index.html;`Arg` 方法=可用属性 https://docs.rs/clap/latest/clap/struct.Arg.html)— https://docs.rs/clap
- serde(序列化)— https://docs.rs/serde
- toml(TOML 编解码)— https://docs.rs/toml
- similar(diff 算法:TextDiff/DiffOp/iter_inline_changes)— https://docs.rs/similar
- anyhow(错误:Context/downcast_ref)— https://docs.rs/anyhow
- unicode-width(显示宽度)— https://docs.rs/unicode-width
