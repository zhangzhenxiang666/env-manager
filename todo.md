# TODO

- [X] 在命令行的em pf create的时候检查profile的名称是否合法
- [X] 在em chek中新增加对profile的名称是否合法的警告
- [X] 创建一个验证体系统一tui模块和命令处理模块的验证(我已经在utils::mod.rs中已经有一个参考示例, 但是不用那么严格, 对于key只要开头不是数字, 然后其他字符都是字母和数字以及下划线, 和'-'即可, profile那么也是)
- [X] 优化tui中的ui显示, 当profiles(显示区域/选择区域)没有元素时要优化ui, 比如居中显示“Not Data", variables部分同理, 目前的tui在(NewAdd模式, Edit模式, 非Edit模式的main_right)
- [X] 在tui模块中新加一个关于main_right的非Edit模式下再创建一个子模式, 计划2个模式, 1. 与目前的非Edit模式相同(默认), 2. 要展示此profile最终的kv列表(各种覆盖后的)
- [ ] 将解析toml格式的错误转换为用户优化的提示
- [ ] 优化release的工具流的构建, 需要在最终的文件中加入版本号, 如果这个更改就需要更改安装脚本的逻辑
- [ ] 参考starship的初始化方式将init命令改造成shell注入命令例如 `eval "$(em init bash)"`, 输出另一个实际命令(防止用户错误使用命令), 另一个命令才是真实的注入指令 类似于eval -- "$(/usr/local/bin/starship init bash --print-full-init)"
