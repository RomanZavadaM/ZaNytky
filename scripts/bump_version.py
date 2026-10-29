#!/usr/bin/env python3
"""Єдина версія проєкту: оновлення або перевірка.

    python scripts/bump_version.py 0.0.5      # записати версію у всі місця
    python scripts/bump_version.py --check    # перевірити, що скрізь одна версія (для CI)

Місця: Cargo.toml, app/package.json, app/src-tauri/Cargo.toml, app/src-tauri/tauri.conf.json,
посилання на випуск і заголовок «(x.y.z)» у README.md та docs/readme/README.*.md.
Лише стандартна бібліотека Python 3.
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VER = r"\d+\.\d+\.\d+"

# (файл, шаблон з однією групою, що містить версію)
MANIFESTS = [
    ("Cargo.toml", r'(?m)^version\s*=\s*"(' + VER + r')"'),
    ("app/package.json", r'(?m)^\s*"version":\s*"(' + VER + r')"'),
    ("app/src-tauri/Cargo.toml", r'(?m)^version\s*=\s*"(' + VER + r')"'),
    ("app/src-tauri/tauri.conf.json", r'(?m)^\s*"version":\s*"(' + VER + r')"'),
]
README_PATTERNS = [
    r"releases/tag/v(" + VER + r")",
    r"\((" + VER + r")\)",   # заголовки «(0.0.4)» у круглих дужках
    r"（(" + VER + r")）",   # японські дужки
]


def read(path):
    with open(os.path.join(ROOT, path), "rb") as f:
        return f.read().decode("utf-8")


def write(path, text):
    with open(os.path.join(ROOT, path), "wb") as f:
        f.write(text.encode("utf-8"))


def readmes():
    return ["README.md"] + sorted(
        os.path.relpath(p, ROOT).replace(os.sep, "/") for p in glob.glob(os.path.join(ROOT, "docs", "readme", "README.*.md"))
    )


def found():
    """Повертає список (файл, версія) для кожного знайденого місця."""
    out = []
    for path, pat in MANIFESTS:
        m = re.search(pat, read(path))
        out.append((path, m.group(1) if m else None))
    for path in readmes():
        text = read(path)
        for pat in README_PATTERNS:
            for m in re.finditer(pat, text):
                out.append((path, m.group(1)))
    return out


def main():
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    args = sys.argv[1:]
    if args == ["--check"]:
        items = found()
        missing = [p for p, v in items if v is None]
        versions = sorted({v for _, v in items if v})
        for p in missing:
            print(f"Не знайдено версії у {p}")
        if len(versions) != 1 or missing:
            print("Версії не збігаються:", ", ".join(versions))
            for p, v in items:
                print(f"  {p}: {v}")
            sys.exit(1)
        print(f"OK: скрізь версія {versions[0]} ({len(items)} місць)")
        return
    if len(args) != 1 or not re.fullmatch(VER, args[0]):
        sys.exit("Використання: bump_version.py X.Y.Z | --check")
    new = args[0]
    for path, pat in MANIFESTS:
        text = read(path)
        m = re.search(pat, text)
        if not m:
            sys.exit(f"Не знайдено версії у {path}")
        text = text[: m.start(1)] + new + text[m.end(1):]
        write(path, text)
    for path in readmes():
        text = read(path)
        for pat in README_PATTERNS:
            text = re.sub(pat, lambda m: m.group(0).replace(m.group(1), new), text)
        write(path, text)
    print(f"Версію змінено на {new}. Перевірте: git diff --stat && python scripts/bump_version.py --check")


if __name__ == "__main__":
    main()
