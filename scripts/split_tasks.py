#!/usr/bin/env python3
"""
900_tasks.md を分割・軽量化するスクリプト

戦略:
1. 全タスクが完了したセクション（##）→ アーカイブに移動
2. 未完了タスクがあるセクション → 完了済みタスク(- [x])と詳細を削除して残す
"""

import re
from pathlib import Path

def is_task_line(line: str) -> tuple[bool, bool]:
    """タスク行かどうかを判定し、(is_task, is_completed)を返す"""
    stripped = line.strip()
    if stripped.startswith("- [x]"):
        return True, True
    if stripped.startswith("- [ ]"):
        return True, False
    return False, False

def get_indent(line: str) -> int:
    """行のインデントレベルを取得"""
    return len(line) - len(line.lstrip())

def process_section(header: str, lines: list[str]) -> tuple[list[str], list[str], bool]:
    """
    セクションを処理して (pending_lines, archived_lines, has_pending) を返す
    """
    pending_lines = []
    archived_lines = []
    has_pending = False
    has_completed = False

    i = 0
    while i < len(lines):
        line = lines[i]
        is_task, is_completed = is_task_line(line)

        if is_task:
            task_indent = get_indent(line)
            task_block = [line]

            # タスクの詳細行（インデントが深い行）を収集
            j = i + 1
            while j < len(lines):
                next_line = lines[j]
                # 空行は含める
                if next_line.strip() == "":
                    task_block.append(next_line)
                    j += 1
                    continue
                # 次の行のインデントがタスクより深ければ詳細
                next_indent = get_indent(next_line)
                if next_indent > task_indent:
                    task_block.append(next_line)
                    j += 1
                else:
                    break

            if is_completed:
                has_completed = True
                archived_lines.extend(task_block)
            else:
                has_pending = True
                pending_lines.extend(task_block)

            i = j
        else:
            # 非タスク行（ヘッダー、説明など）
            pending_lines.append(line)
            archived_lines.append(line)
            i += 1

    return pending_lines, archived_lines, has_pending

def main():
    docs_dir = Path(__file__).parent.parent / "docs"
    tasks_file = docs_dir / "900_tasks.md"
    archive_file = docs_dir / "901_tasks_archived.md"

    content = tasks_file.read_text(encoding="utf-8")
    lines = content.split("\n")

    # アーカイブヘッダー
    archived_output = [
        "# 完了済みタスクアーカイブ",
        "",
        "> このファイルは `docs/900_tasks.md` から移動した完了済みタスクの履歴です。",
        "",
    ]

    # メインファイルヘッダー
    pending_output = [
        "# タスク分解・進捗管理",
        "",
        "> 完了済みタスクは `docs/901_tasks_archived.md` を参照",
        "",
    ]

    # セクション単位で処理
    current_header = ""
    current_lines = []
    sections_pending = 0
    sections_archived = 0

    for line in lines:
        if line.startswith("## "):
            # 前のセクションを処理
            if current_header:
                pending, archived, has_pending = process_section(current_header, current_lines)
                if has_pending:
                    pending_output.append(current_header)
                    pending_output.extend(pending)
                    sections_pending += 1
                else:
                    archived_output.append(current_header)
                    archived_output.extend(archived)
                    sections_archived += 1

            current_header = line
            current_lines = []
        elif line.startswith("# "):
            # タイトル行はスキップ（ヘッダーで置き換え）
            continue
        else:
            current_lines.append(line)

    # 最後のセクションを処理
    if current_header:
        pending, archived, has_pending = process_section(current_header, current_lines)
        if has_pending:
            pending_output.append(current_header)
            pending_output.extend(pending)
            sections_pending += 1
        else:
            archived_output.append(current_header)
            archived_output.extend(archived)
            sections_archived += 1

    # 空行の重複を削除
    def clean_empty_lines(lines):
        result = []
        prev_empty = False
        for line in lines:
            is_empty = line.strip() == ""
            if is_empty and prev_empty:
                continue
            result.append(line)
            prev_empty = is_empty
        return result

    pending_output = clean_empty_lines(pending_output)
    archived_output = clean_empty_lines(archived_output)

    # ファイル書き出し
    tasks_file.write_text("\n".join(pending_output), encoding="utf-8")
    archive_file.write_text("\n".join(archived_output), encoding="utf-8")

    print(f"分割完了:")
    print(f"  900_tasks.md: {sections_pending} セクション, {len(pending_output)} 行")
    print(f"  901_tasks_archived.md: {sections_archived} セクション, {len(archived_output)} 行")

if __name__ == "__main__":
    main()
