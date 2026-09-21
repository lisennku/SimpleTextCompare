# Simple Text Compare 简单文本比较

# 使用方法

## 初始化

在使用程序之前，需要先进行初始化，生成配置文件后，才可以继续使用
`stc conf --init`

## 配置

- 获取配置
  - `stc conf --list`
- 设置列宽
  - `stc conf --code-width xx` 代码列宽
  - `stc conf --no-width xx` 行号列宽
- 设置`less`路径
  - `stc conf --less-path xx`
- 设置`inline` 是否启用行内对比
  - `stc conf --inline` 设置为`true`
  - `stc conf --inline=false` 设置为`false`
- 需要注意
  - `--list`/`--init`互斥，但是可以结合另外三个进行设置
    - `--init`结合其他，先初始化后更改
    - `--list`结合其他，先修改后展示

## 对比

`stc diff left_file right_file [--path file_dir] [-p file_dir]`

- `--path`/`-p`参数说明
  - 当两个文件在同一路径内，可以使用该参数指定路径，避免`left_file`和`right_file`输入太长

# 终端显示说明

当前版本不会刷新终端显示，会在执行命令后在下方直接显示，或者转移到`less`显式

- `ANSI`颜色渲染会判断是否在终端内，如果重定向则忽略`ANSI`染色

# to-do

- [x] 宽度设置`width`
  - 使用`stc conf`功能进行设置
  - 配置表保存于`~/stc_conf/stc.toml`
- [x] 输出长度控制，启用`--less`参数，将输出转移到`less`，需提前安装否则会导致`panic`
- [x] 颜色标记
- [x] 行内对比
- [x] 增加类`git diff`输出
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
- `serde`
    - 序列化
- `toml`
    - 配合`serde`进行序列化





