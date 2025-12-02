# 项目介绍

这是一个用于管理环境变量的命令行工具, 采用rust编写
这个项目的最终的使用是在.bashrc中添加一个函数来配合rust编译生成的二进制文件来达到操作环境变量的目的:
脚本函数如下:

```bash
function em() {
    local output
    local exit_code

    output=$(/home/zzx/Codespace/rust_code/env-manage/target/debug/env-manage "$@")
    exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        return $exit_code
    fi

    if [[ -z "$output" ]]; then
        return 0
    fi

    if ! eval "$output" 2>/dev/null; then
        echo "$output"
        return $exit_code
    fi
}
```

如果要操作当前环境变量, rust二进制文件会往标准输出中输出shell命令, 而eval函数会尝试解析输出的命令并执行
如果命令执行失败, 会忽略eval的错误, 并返回ouput的输出和退出码, 这是因为这个命令行工具有帮助信息, 因此除非rust中返回了错误码, 否则不会将其视为错误

## 计划

项目目前采用了模块化的设计, cli模块存放了命令解析逻辑(clap构建)
在handles模块是处理各个命令的逻辑, 其中mod.rs中有run函数负责分发个命令的处理逻辑
其他子模块则各自处理需要处理的情况

core模块计划存放公共可用的逻辑

config模块里面定义了需要的模型和数据结构, 也有ConfigManager来管理路径

整个项目的思路是这样的:
用户可以定义一组环境变量(在项目中称为profile, 以toml的格式存在), 在项目最终的使用中会在~/.config/env-manage中创建profiles目录。

而profile的格式如下:

```toml
profiles = [] # 用来表示该profile引用了其他哪些profile
[variables] # 单独定义的环境变量 k = v这样的结构
```

现在tui模块还没有实现你也不要管
目前主要是将handles模块的其他命令解析的开发
