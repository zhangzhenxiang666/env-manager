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
em init
```

如果要操作当前环境变量, rust二进制文件会往标准输出中输出shell命令, 而eval函数会尝试解析输出的命令并执行
如果命令执行失败, 会忽略eval的错误, 并返回ouput的输出和退出码, 这是因为这个命令行工具有帮助信息, 因此除非rust中返回了错误码, 否则不会将其视为错误
况

config模块里面定义了需要的模型和数据结构, 也有ConfigManager来管理路径

整个项目的思路是这样的:
用户可以定义一组环境变量(在项目中称为profile, 以toml的格式存在), 在项目最终的使用中会在~/.config/env-manage中创建profiles目录。

而profile的格式如下:

```toml
profiles = [] # 用来表示该profile引用了其他哪些profile
[variables] # 单独定义的环境变量 k = v这样的结构
```

## TUI 核心设计

TUI 的实现遵循了受 [Ratatui Template](https://github.com/ratatui-org/ratatui-template) 启发的组件化、模块化的设计思想，以确保代码的清晰性、可扩展性和可维护性。

### 1. 组件化架构 (Component-Based Architecture)

- **`App` 作为容器**: 主 `App` 结构体 (`tui/app.rs`) 是一个顶层容器，它持有各个独立的业务组件，并管理应用的全局状态（如 `AppState`, `shutdown`）。它不直接处理具体业务逻辑，而是将任务委托给相应的组件。
- **独立的业务组件**: 每个主要的 UI 功能或区域都被封装在一个独立的组件中（位于 `tui/components/` 目录），例如：
  - `ListComponent`: 负责主界面的 `profile` 列表的显示、选择、滚动和变更追踪。
  - `AddNewComponent`: 负责“新建 Profile”弹窗内的所有状态，包括输入框、焦点管理和未来可扩展的继承列表。
- **单一职责**: 每个组件都遵循单一职责原则，只关心自己的状态和行为。

### 2. 可复用的 `Input` 组件

- **核心输入单元**: 为了处理所有文本输入场景（如新建名称、搜索、过滤），我们抽象出了一个可复用的 `Input` 结构体 (`tui/utils.rs`)。
- **封装的状态与逻辑**: `Input` 结构体是所有文本输入框的“单一数据源”。它封装了以下内容：
  - **状态**: `text` (输入文本), `cursor_position` (光标位置), `is_valid` (校验状态), `error_message` (错误信息)。
  - **逻辑**: 所有与输入和光标操作相关的方法，如 `move_cursor_left/right`, `enter_char`, `delete_char` 等。
- **优势**: 这种设计避免了在多个模块中重复实现输入框逻辑，并提供了统一、健壮的输入体验。

### 3. 清晰的数据流与事件处理

- **事件路由**: 主事件循环 (`tui/event.rs`) 接收到键盘事件后，会根据当前的 `AppState` 将事件路由到对应的事件处理模块（如 `event/add_new.rs`）。
- **组件方法调用**: 事件处理模块是“薄”的，它只负责解析按键，并调用相应组件的**方法**来更新状态。
  - **示例**: 在 `AddNew` 状态下，`add_new.rs` 接收到 `Left` 键，它会调用 `app.add_new_component.name_input.move_cursor_left()`。它不直接操作 `cursor_position`。
- **视图 = 状态的函数**: 渲染模块 (`tui/widgets/`) 是“无状态”的。它们只负责接收组件的引用，读取其内部状态，并将 UI 绘制到屏幕上。它们不包含任何业务逻辑。

这个架构使得添加新功能（比如“重命名”或“编辑”弹窗）变得非常简单：只需创建一个新的组件，为其实现状态和方法，然后在 `app.rs` 中注册，并添加相应的事件处理和渲染逻辑即可。

## TUI 功能实现状态

- [x] **基本布局**: 实现了 Header, Main (左右分栏), Bottom 的三段式布局。
- [x] **Profile 列表显示**: 左侧面板能成功加载并显示所有 `profile`。
- [x] **列表导航**: 可使用 `J`/`K`/`↑`/`↓` 在 `profile` 列表中进行上下导航。
- [x] **内容显示**: 右侧面板能显示选中 `profile` 的“继承列表”和“环境变量”。
- [x] **状态切换**:
  - `List -> Edit`: 按 `Enter` 可进入 `Edit` 状态，右侧面板边框高亮。
  - `Edit -> List`: 在 `Edit` 状态下按 `Esc` 可返回 `List` 状态。
  - `List -> Delete`: 按 `d` 可进入删除确认状态。
- [x] **变更追踪**: 对 `profile` 的修改会被追踪，并在列表项旁用 `*` 标记。
- [x] **保存功能**:
  - `s` (Save Selected): 可保存当前选中的、已修改的 `profile`。
  - `w` (Save All): 可保存所有已修改的 `profile`。
- [x] **安全删除功能**:
  - `d` (Delete): 删除 `profile` 前会进行依赖检查，并弹出确认窗口。
- [x] **新建功能 `n`**:
  - 弹出窗口并允许用户输入新 `profile` 的名称，支持光标左右移动。
  - **交互优化**: `Enter` 键自动切换焦点，输入错误时阻止切换；`Esc` 安全退出。
  - **变量编辑**: 支持添加、删除、编辑变量 (`Key`/`Value`)。
  - **输入校验**: 实时校验 Profile 名称和变量 Key（非空、无空格、不以数字开头）。
  - **UI 优化**: 变量列表支持滚动条，高度自适应，标题显示计数。
  - **代码重构**: 事件处理逻辑 (`tui/event/add_new.rs`) 已重构，逻辑更清晰。
- [x] **重命名功能 `F2`**: 对选中的 `profile` 进行重命名。
- [x] **搜索功能 `/`**: 允许用户通过名称搜索 `profile`。
- [x] **编辑模式**: 在 `Edit` 状态下，实现对“继承列表”和“环境变量”的增删改。

## 现阶段计划

- 优化ConfigManager的创建逻辑
  目前实现在创建一个ConfigManager实例时会读取整个profiles目录下的toml文件, 以前这样做的目的是为了构建图来检查是否有循环依赖关系,
  这要做的好处是可以在每个命令执行前就发现错误并提醒用户, 坏处是如果文件过多, 每次执行命令都要读取所有文件, 影响性能
  计划将ConfigManager的new方法改为创建一个空实例, 提供一种懒加载的方式来获取文件内容,
  要提供提供一个方法, 这个方法接收一个profile名称
  拿到profile名称后读取文件内容, 首先将读取到的文件保存到appconfig中的profiles的哈希表中, 然后添加一个节点,
  后面开始以dfs的方式来遍历profiles字段的内容来构建图, 方式和上面一致, 读取文件加入节点, 添加指向这个这个节点的边, 直到传入的profile的直接依赖和间接依赖都解析完毕(实际上是创建了一个以传入名称为根的树, 相比原来的直接创建图会快很多)
  在这个过程中也许会有2个错误, 首先是文件不存在, 第二个是也许会出现循环依赖(通过dag的插入边可以检查到是否有循环依赖), 这个时候我要告诉用户对应的详细错误信息后返回即可
  这个优化过后, 在em profile add ... 命令中只需要加载对应的profile文件, 然后构建树, 然后再尝试加入边(当然需要先判断是要加入的文件是否存在),
  再profile delete中就不需要判断是否有其他profile引用了这个profile, 因为在适当的时候用户会得知错误的信息的

- 创建检查命令
  既然我们采用了上面的懒加载方式, 就说明了用户创建的profiles目录中可能会有错误, 或文件不存在或循环依赖,
  我们要新创建一个命令, 这个命令的功能是检查当前的profiles目录中的所有文件, 它要向用户报告当前的profiles中的所有的错误(要非常准确与合理)

- 创建修复命令
  既然我们的profiles中可能会出现2个错误
  1. 文件不存在
  2. 循环依赖
  那么我们就要创建一个修复命令, 这个命令的功能是修复profiles中的错误, 它会自动修改那些在profiles字段中引用不存在的profile的profiles字段的(就是删除profiles字段中不存在的item)
  对于循环依赖我们可以A->B->C->A我们会将C->A的边删除(通过修改C的profiles字段)来完成修复
  然后要向用户报告我们干了什么
