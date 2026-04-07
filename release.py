#!/usr/bin/env python3
"""自动同步版本号并创建 Git tag 发布。

用法：
    python release.py           # 自动递增版本号（0.0.1 → 0.0.2，0.0.9 → 0.1.0）
    python release.py 0.1.0     # 指定版本号
    python release.py 1.0.0-beta.1
"""

import re
import subprocess
import sys
import os


def run(cmd, check=True):
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if check and result.returncode != 0:
        print(f"错误: {cmd}\n{result.stderr.strip()}")
        sys.exit(1)
    return result.stdout.strip()


def read_current_version(filepath):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()
    m = re.search(r'^version\s*=\s*"([^"]*)"', content, re.MULTILINE)
    if not m:
        print(f"错误: {filepath} 中未找到 version 字段")
        sys.exit(1)
    return m.group(1)


def bump_version(version):
    m = re.match(r'^(\d+)\.(\d+)\.(\d+)', version)
    if not m:
        print(f"错误: 无法解析版本号 '{version}'")
        sys.exit(1)
    major, minor, patch = int(m.group(1)), int(m.group(2)), int(m.group(3))
    patch += 1
    if patch > 9:
        patch = 0
        minor += 1
    if minor > 9:
        minor = 0
        major += 1
    return f"{major}.{minor}.{patch}"


def update_cargo_toml(filepath, version):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    new_content = re.sub(
        r'^version\s*=\s*"[^"]*"',
        f'version = "{version}"',
        content,
        count=1,
        flags=re.MULTILINE,
    )

    if new_content == content:
        print(f"警告: {filepath} 版本号未变更")
        return

    with open(filepath, "w", encoding="utf-8") as f:
        f.write(new_content)
    print(f"  已更新 {filepath}")


def main():
    root = os.path.dirname(os.path.abspath(__file__))
    cargo_toml = os.path.join(root, "Cargo.toml")

    if len(sys.argv) > 2:
        print(f"用法: python {sys.argv[0]} [version]")
        print(f"示例: python {sys.argv[0]}           # 自动递增")
        print(f"      python {sys.argv[0]} 0.1.0     # 指定版本号")
        sys.exit(1)

    if len(sys.argv) == 2:
        version = sys.argv[1].lstrip("v")
    else:
        current = read_current_version(cargo_toml)
        version = bump_version(current)
        print(f"当前版本: {current} → 新版本: {version}")

    tag = f"v{version}"

    # 检查工作区是否干净
    status = run("git status --porcelain", check=False)
    if status:
        print("存在未提交的更改，请先提交或暂存：")
        print(status)
        sys.exit(1)

    # 检查 tag 是否已存在
    tags = run("git tag -l", check=False)
    if tag in tags.splitlines():
        print(f"标签 {tag} 已存在，请先删除：git tag -d {tag} && git push origin --delete {tag}")
        sys.exit(1)

    print(f"准备发布 {tag} ...")

    # 更新版本号
    update_cargo_toml(cargo_toml, version)

    # 提交版本号变更
    run("git add Cargo.toml")
    run(f'git commit -m "chore: bump version to {version}"')

    # 创建 tag
    run(f"git tag {tag}")

    # 推送
    print(f"推送代码和标签 {tag} ...")
    run("git push origin master")
    run(f"git push origin {tag}")

    print(f"\n发布完成！{tag}")


if __name__ == "__main__":
    main()
