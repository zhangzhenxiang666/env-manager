# 环境变量管理器 (env-manage)

[![English Documentation](https://img.shields.io/badge/Docs-English-blue.svg)](README.md)

一个用 Rust 编写的命令行工具，用于管理您的 Shell 环境变量。它支持在不同的配置文件（Profiles）之间切换，并自动处理依赖关系。同时，它也提供了一个 TUI（终端用户界面）来进行可视化管理。

```bash
em -h  # 显示帮助信息
```

![帮助信息](./assets/help.png)

此工具使用 TOML 格式存储环境变量配置，配置文件默认存储在 `~/.config/env-manage/profiles` 目录下。

- **`global.toml`**: 这是一个特殊的配置文件，位于 `~/.config/env-manage/global.toml`，它将在每次终端启动时自动加载。

每一个 Profile 包含两个部分：

- **variables**: 一个键值对列表，表示需要设置的环境变量。
- **profiles**: 一个列表，包含当前 Profile 依赖的其他 Profile。

配置文件示例：

```toml
profiles = []

[variables]
https_proxy = "http://172.26.240.1:7890"
all_proxy = "http://172.26.240.1:7890"
http_proxy = "http://172.26.240.1:7890"
```

## 功能特性

- **配置文件管理**: 创建并将环境变量组织成不同的配置文件。
- **依赖解析**: 处理配置文件之间复杂的依赖关系。
- **TUI 界面**: 终端用户界面，用于直观地管理配置。
- **Shell Integration**: 支持 Bash, Zsh, Fish, 和 PowerShell。

## 安装指南

目前为以下 Shell 提供了自动安装脚本。这些脚本将下载最新的二进制文件并配置您的 Shell 环境。

### Bash

```bash
curl -fsSL https://raw.githubusercontent.com/zhangzhenxiang666/env-manager/main/scripts/install_bash.sh | bash
```

### Zsh

```bash
curl -fsSL https://raw.githubusercontent.com/zhangzhenxiang666/env-manager/main/scripts/install_zsh.sh | bash
```

### Fish

```bash
curl -fsSL https://raw.githubusercontent.com/zhangzhenxiang666/env-manager/main/scripts/install_fish.sh | bash
```

### PowerShell (Windows)

```powershell
irm https://raw.githubusercontent.com/zhangzhenxiang666/env-manager/main/scripts/install_powershell.ps1 | iex
```

### 手动安装

如果您更喜欢手动安装：

1. 从 [Releases](https://github.com/zhangzhenxiang666/env-manager/releases) 页面下载二进制文件。
2. 将其放置在您选择的目录中。**注意**：您**不需要**将此目录添加到系统的 `PATH` 环境变量中。
3. 将初始化命令添加到您的 Shell 配置文件中。请将 `/path/to/env-manage` 替换为您实际的二进制文件路径。

    - **Bash (`~/.bashrc`)**:

        ```bash
        eval "$(/path/to/env-manage init bash)"
        ```

    - **Zsh (`~/.zshrc`)**:

        ```bash
        eval "$(/path/to/env-manage init zsh)"
        ```

    - **Fish (`~/.config/fish/config.fish`)**:

        ```fish
        /path/to/env-manage init fish | source
        ```

    - **PowerShell (`$PROFILE`)**:

        ```powershell
        Invoke-Expression (& "C:\path\to\env-manage.exe" init powershell | Out-String)
        ```

> **注意**: 安装完成后，请重启您的终端或运行 `source ~/.bashrc` (或相应配置文件) 以加载配置。

## 使用方法

### TUI 管理界面

运行以下命令启动 TUI 管理界面：

```bash
em ui
```

![TUI](./assets/tui.png)

### 常用命令

- **临时加载环境变量**:

    在当前会话中加载指定的 Profile 或直接设置变量。

    ```bash
    em use <profile_name or key=value>...
    # 或者使用别名: em activate
    ```

    示例: `em use profile1 profile2 http_proxy=http://172.26.240.1:7890`

- **卸载环境变量**:

    从当前会话中移除指定的 Profile 或变量。

    ```bash
    em unuse <profile_name or key>...
    # 或者使用别名: em deactivate, em drop
    ```

    示例: `em unuse profile1 profile2 http_proxy`

- **检查状态**:

    检查当前环境的状态和一致性。

    ```bash
    em check
    ```

- **修复一致性问题**:

    尝试修复环境变量配置中的不一致问题。

    ```bash
    em fix
    ```

## 配置

默认情况下，配置文件位于 `~/.config/env-manage/profiles` 目录。
