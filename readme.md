# Simple Text Compare 简单文本比较

- 基础使用
  `stc left_file right_file` 逐行显示文本差异
- 设置统一目录
  `stc left_file right_file -p | --path dir_path`

# 终端显示说明

当前版本不会刷新终端显示，会在执行命令后在下方直接显示，类似`cat`，但是没有控制输出长度

# to-do

- [ ] 宽度设置`width`
- [ ] 输出长度控制，类似`cat`
- [ ] 差异类型限制
- [ ] 新增终端显示

# related crates

- `clap`
    - 用于命令行参数设置与解析
- `similar`
    - 文本比较
- `anyhow`
    - 统一错误处理
- `unicode_width`
    - 字符宽度处理，用于输出

# 开发笔记

- 函数参数的按值传递

  函数中的按值传递，对于非`Copy`类型的，自然发生所有权移动，而不是必须显式说明
  因此，类似`Vec`的`push(value: T)`方法，会将`value`的所有权转移

-  `move`发生后，只要马上重新赋值，后面又可以继续用这个变量名

  ```rust
  lines.push(current_chars);
  current_chars = String::new();
  ```

  - 也可以使用`std::mem::take(&mut var)`来把里面的字符串拿走，同时把原变量留空

- 测试显示输出

  - `cargo test -- --no-capture`  实时显示测试过程里的输出

  - `cargo test -- --show-output` 等所有测试跑完后，把成功测试的输出也集中显示出来







