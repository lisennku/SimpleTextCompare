# Simple Text Compare 简单文本比较

- 基础使用 逐行显示文本差异
    - `stc left_file right_file`
- 设置统一目录
    - `stc left_file right_file -p dir_path`
- 设置代码列宽度
    - `stc left_file right_file --code-width 60`
- 设置行号列宽度
    - `stc left_file right_file --no-width 5`

# 终端显示说明

当前版本不会刷新终端显示，会在执行命令后在下方直接显示，类似`cat`，但是没有控制输出长度

# to-do

- [x] 宽度设置`width`
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

- `move`发生后，只要马上重新赋值，后面又可以继续用这个变量名

  ```rust
      lines.push(current_chars);
      current_chars = String::new();
  ```

  - 也可以使用`std::mem::take(&mut var)`来把里面的字符串拿走，同时把原变量留空

- `clap`中`arg`的属性`value_parser`，可以传入一个函数，用于值的解析与限定

  - 函数的参数是`&str`类型，返回值是`Result<dtype, String>`

- `Option`的`map`，对`Some`里的值进行闭包处理，但是不处理`None`

- `Option`的`and_then`，对`Some`里的值进行闭包处理，并且闭包必须返回`Option`

  - `and_then`函数会将`Option<Option<>>`拍平为`Option`

- `Result`的`map_err`，转换错误，但不改变成功值

- `str`进行解析到数字时，需要显式指定具体类型`s.parse::<usize>()`

  - 可以自动推断时除外


- 测试显示输出

    - `cargo test -- --no-capture`  实时显示测试过程里的输出

    - `cargo test -- --show-output` 等所有测试跑完后，把成功测试的输出也集中显示出来







