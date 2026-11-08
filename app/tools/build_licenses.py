"""Writes licenses/THIRD-PARTY.txt: every Rust crate built into the program, with its license texts.

Run from app/: python tools/build_licenses.py
Only crates that end up in the Windows executable are listed (no dev dependencies).
"""
import hashlib
import json
import subprocess
from pathlib import Path

TARGET = "x86_64-pc-windows-msvc"
OWN = {"astro-core", "kosmogram"}
LICENSE_FILES = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE", "COPYRIGHT")

meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--format-version", "1", "--filter-platform", TARGET, "--locked"],
    capture_output=True, check=True, text=True, encoding="utf-8").stdout)
packages = {p["id"]: p for p in meta["packages"]}
nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}

root = next(pid for pid, p in packages.items() if p["name"] == "kosmogram")
seen, stack = set(), [root]
while stack:
    pid = stack.pop()
    if pid in seen:
        continue
    seen.add(pid)
    for dep in nodes[pid]["deps"]:
        if any(k["kind"] != "dev" for k in dep["dep_kinds"]):
            stack.append(dep["pkg"])

crates = sorted((packages[pid] for pid in seen if packages[pid]["name"] not in OWN), key=lambda p: (p["name"], p["version"]))

texts = {}  # hash -> (text, [crate names])
listing = []
missing = []
for p in crates:
    folder = Path(p["manifest_path"]).parent
    files = sorted(f for f in folder.iterdir() if f.is_file() and f.name.upper().startswith(LICENSE_FILES))
    if p.get("license_file"):
        extra = folder / p["license_file"]
        if extra.is_file() and extra not in files:
            files.append(extra)
    label = f'{p["name"]} {p["version"]}'
    listing.append(f'{label:40} {p.get("license") or "viz text licence":30} {p.get("repository") or ""}')
    if not files:
        missing.append(label)
    for f in files:
        text = f.read_text(encoding="utf-8", errors="replace").strip()
        key = hashlib.sha256(text.encode()).hexdigest()
        texts.setdefault(key, (text, []))[1].append(f"{label} ({f.name})")

out = [
    "Knihovny třetích stran sestavené do programu Kosmogram",
    "=======================================================",
    "",
    "Program obsahuje níže uvedené knihovny jazyka Rust a frameworku Tauri. Každá je pod licencí",
    "uvedenou v seznamu. Úplné texty licencí a upozornění na autorská práva následují za seznamem.",
    "",
    f"Počet knihoven: {len(crates)}",
    "",
    *listing,
]
if missing:
    out += ["", "Knihovny bez souboru s licencí v balíčku (licence podle údaje v seznamu):", *missing]
for text, users in texts.values():
    out += ["", "-" * 78, "Platí pro: " + ", ".join(users), "-" * 78, "", text]

dest = Path(__file__).resolve().parent.parent / "licenses" / "THIRD-PARTY.txt"
dest.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
print(f"{len(crates)} knihoven, {len(texts)} různých textů, bez souboru: {len(missing)} -> {dest}")
