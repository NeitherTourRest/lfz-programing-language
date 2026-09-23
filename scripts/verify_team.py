#!/usr/bin/env python3
"""LFZ 项目团队配置验收脚本 (acceptance harness for the 14-agent team).

Python 3.13, standard library only (no pip installs). PyYAML is used when
available; otherwise a minimal fallback parser handles the flat
``key: value`` frontmatter plus the single nested ``permission:`` map.

Run from the project root::

    python scripts/verify_team.py

Exit code is 0 only when every check passes.

Scope note: this harness checks STRUCTURE ONLY (file set, frontmatter fields,
path cross-references, line budget). It is NOT a semantic acceptance test and
does not validate prompt content, rubric mapping, or protocol consistency.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

try:  # optional dependency; fallback parser below covers its absence
    import yaml  # type: ignore[import-not-found]

    _HAS_YAML = True
except ImportError:  # pragma: no cover - depends on the environment
    _HAS_YAML = False

ROOT = Path(__file__).resolve().parent.parent
AGENTS_DIR = ROOT / ".opencode" / "agents"
TEAM_AGENTS_DIR = ROOT / ".opencode" / "team" / "agents"
CONFIG_PATH = ROOT / ".opencode" / "opencode.json"
AGENTS_MD_PATH = ROOT / "AGENTS.md"

AGENT_NAMES: tuple[str, ...] = (
    "team-lead",
    "requirements-analyst",
    "language-architect",
    "core-dev",
    "runtime-dev",
    "tooling-dev",
    "test-engineer",
    "perf-engineer",
    "verifier",
    "docs-writer",
    "ai-dx-engineer",
    "app-dev",
    "release-manager",
    "ppt-presenter",
)

VALID_MODES: frozenset[str] = frozenset({"primary", "subagent", "all"})
REQUIRED_MODEL: str = "deepseek/deepseek-flash"
ALLOWED_PERMISSION_KEYS: frozenset[str] = frozenset(
    {
        "read",
        "edit",
        "glob",
        "grep",
        "list",
        "bash",
        "task",
        "external_directory",
        "todowrite",
        "question",
        "webfetch",
        "websearch",
        "lsp",
        "skill",
        "doom_loop",
    }
)
BUILTIN_AGENT_NAMES: tuple[str, ...] = (
    "sisyphus",
    "hephaestus",
    "prometheus",
    "atlas",
    "sisyphus-junior",
    "metis",
    "momus",
    "oracle",
    "librarian",
    "explore",
    "multimodal-looker",
    "build",
    "plan",
    "general",
    "compaction",
    "summary",
    "title",
)
TEAM_PATH_RE = re.compile(r"\.opencode/team/[A-Za-z0-9_\-./<>]+")


def extract_frontmatter(text: str) -> str | None:
    """Return the block between the leading ``---`` pair, or None."""
    lines = text.splitlines()
    if not lines:
        return None
    if lines[0].lstrip("\ufeff").strip() != "---":
        return None
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            return "\n".join(lines[1:index])
    return None


def _strip_inline_comment(value: str) -> str:
    return re.sub(r"\s+#.*$", "", value).rstrip()


def _parse_scalar(value: str) -> Any:
    value = _strip_inline_comment(value)
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
        return value[1:-1]
    lowered = value.lower()
    if lowered in {"true", "yes", "on"}:
        return True
    if lowered in {"false", "no", "off"}:
        return False
    if lowered in {"null", "~", ""}:
        return None
    for convert in (int, float):
        try:
            return convert(value)
        except ValueError:
            continue
    return value


def parse_frontmatter_fallback(text: str) -> dict[str, Any]:
    """Minimal parser: flat ``key: value`` + one nested ``permission:`` map."""
    result: dict[str, Any] = {}
    current_map: dict[str, Any] | None = None
    for raw in text.splitlines():
        line = raw.rstrip()
        stripped = line.lstrip()
        if not stripped or stripped.startswith("#"):
            continue
        indent = len(line) - len(stripped)
        if ":" not in stripped:
            continue
        key, _, value = stripped.partition(":")
        key = key.strip()
        value = value.strip()
        if indent == 0:
            if value == "":
                current_map = {}
                result[key] = current_map
            else:
                current_map = None
                result[key] = _parse_scalar(value)
        elif current_map is not None:
            current_map[key] = _parse_scalar(value)
    return result


def parse_frontmatter(text: str) -> dict[str, Any]:
    """Parse frontmatter via PyYAML when available, else the fallback."""
    if _HAS_YAML:
        data = yaml.safe_load(text)
        if data is None:
            return {}
        if not isinstance(data, dict):
            raise ValueError("frontmatter must be a YAML mapping")
        return data
    return parse_frontmatter_fallback(text)


def load_agent(name: str) -> tuple[dict[str, Any] | None, str | None, str | None]:
    """Return (frontmatter, full_text, error) for one agent file."""
    path = AGENTS_DIR / f"{name}.md"
    if not path.is_file():
        return None, None, None
    text = path.read_text(encoding="utf-8", errors="replace")
    fm_text = extract_frontmatter(text)
    if fm_text is None:
        return None, text, "missing frontmatter (no leading --- ... --- block)"
    try:
        return parse_frontmatter(fm_text), text, None
    except Exception as exc:  # report the file, never crash the harness
        return None, text, f"frontmatter parse error: {exc}"


def normalize_name(name: str) -> str:
    return re.sub(r"[-_]", "", name).lower()


def _permission_map(fm: dict[str, Any]) -> dict[str, Any]:
    perm = fm.get("permission")
    return perm if isinstance(perm, dict) else {}


def _perm_task(fm: dict[str, Any]) -> str | None:
    value = _permission_map(fm).get("task")
    return str(value) if value is not None else None


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _names(names: Any) -> str:
    return ", ".join(sorted(names)) if names else "-"


def main() -> int:
    results: list[tuple[str, bool, str]] = []

    def record(check_id: str, ok: bool, detail: str) -> None:
        results.append((check_id, ok, detail))

    # ---- 01: .opencode/agents/ contains exactly the 14 expected files
    actual_files: list[str] = []
    if AGENTS_DIR.is_dir():
        actual_files = sorted(p.name for p in AGENTS_DIR.iterdir() if p.is_file())
    expected_files = {f"{name}.md" for name in AGENT_NAMES}
    missing_files = sorted(expected_files - set(actual_files))
    extra_files = sorted(set(actual_files) - expected_files)
    record(
        "01-agents-exact-14-files",
        not missing_files and not extra_files,
        f"found={len(actual_files)} missing={_names(missing_files)} extra={_names(extra_files)}",
    )

    # ---- load all agents once (frontmatter, raw text, parse errors)
    data: dict[str, dict[str, Any]] = {}
    text_map: dict[str, str] = {}
    errors: dict[str, str] = {}
    for name in AGENT_NAMES:
        fm, text, err = load_agent(name)
        if text is not None:
            text_map[name] = text
        if fm is not None:
            data[name] = fm
        if err is not None:
            errors[name] = err

    missing_from_disk = sorted(set(AGENT_NAMES) - set(text_map))
    unverifiable = sorted(set(errors) | set(missing_from_disk))

    # ---- 02: frontmatter parseable
    detail = " ".join(f"{name}={errors[name]}" for name in sorted(errors)) or "all parseable"
    record("02-frontmatter-parseable", not unverifiable, detail)

    # ---- 03: description present and non-empty
    bad_desc = [
        name
        for name, fm in data.items()
        if not (isinstance(fm.get("description"), str) and fm["description"].strip())
    ]
    record(
        "03-description-nonempty",
        not bad_desc and not unverifiable,
        f"bad={_names(bad_desc)} unverifiable={_names(unverifiable)}",
    )

    # ---- 04: mode in {primary, subagent, all}
    bad_mode = [name for name, fm in data.items() if fm.get("mode") not in VALID_MODES]
    record(
        "04-mode-valid-enum",
        not bad_mode and not unverifiable,
        f"bad={_names(bad_mode)} unverifiable={_names(unverifiable)}",
    )

    # ---- 05: model == deepseek/deepseek-flash
    bad_model = [name for name, fm in data.items() if fm.get("model") != REQUIRED_MODEL]
    record(
        "05-model-deepseek-v4-flash",
        not bad_model and not unverifiable,
        f"bad={_names(bad_model)} unverifiable={_names(unverifiable)}",
    )

    # ---- 06: no `name:` key (parsed dict and raw top-level lines)
    bad_name = [name for name, fm in data.items() if "name" in fm]
    for name, text in text_map.items():
        fm_text = extract_frontmatter(text)
        if fm_text and re.search(r"^name\s*:", fm_text, re.MULTILINE):
            bad_name.append(name)
    bad_name = sorted(set(bad_name))
    record(
        "06-no-name-frontmatter",
        not bad_name and not unverifiable,
        f"bad={_names(bad_name)} unverifiable={_names(unverifiable)}",
    )

    # ---- 07: temperature present and numeric
    bad_temp = [name for name, fm in data.items() if not _is_number(fm.get("temperature"))]
    record(
        "07-temperature-numeric",
        not bad_temp and not unverifiable,
        f"bad={_names(bad_temp)} unverifiable={_names(unverifiable)}",
    )

    # ---- 08: team-lead mode primary AND permission.task allow
    tl = data.get("team-lead")
    tl_ok = tl is not None and tl.get("mode") == "primary" and _perm_task(tl) == "allow"
    if tl is None:
        tl_detail = "team-lead missing or unparseable"
    else:
        tl_detail = (
            f"mode={tl.get('mode')!r} permission.task={_perm_task(tl)!r} (need primary/allow)"
        )
    record("08-team-lead-primary-task-allow", tl_ok, tl_detail)

    # ---- 09: other 13 agents permission.task deny
    bad_deny = [
        name
        for name in AGENT_NAMES
        if name != "team-lead" and (name not in data or _perm_task(data[name]) != "deny")
    ]
    record(
        "09-others-task-deny",
        not bad_deny,
        f"bad={_names(bad_deny)}",
    )

    # ---- 10: all permission keys within the allowed set
    used_keys: set[str] = set()
    non_dict: list[str] = []
    for name, fm in data.items():
        perm = fm.get("permission")
        if perm is None:
            continue
        if isinstance(perm, dict):
            used_keys.update(str(key) for key in perm)
        else:
            non_dict.append(name)
    invalid_keys = sorted(used_keys - ALLOWED_PERMISSION_KEYS)
    ok_keys = bool(data) and not invalid_keys and not non_dict
    record(
        "10-permission-keys-allowed",
        ok_keys,
        f"used={_names(used_keys)} invalid={_names(invalid_keys)} non_dict={_names(non_dict)}",
    )

    # ---- 11: .opencode/opencode.json valid + $schema + default_agent
    if not CONFIG_PATH.is_file():
        record("11-opencode-json", False, ".opencode/opencode.json missing")
    else:
        try:
            cfg = json.loads(CONFIG_PATH.read_text(encoding="utf-8", errors="replace"))
        except json.JSONDecodeError as exc:
            record("11-opencode-json", False, f"invalid JSON: {exc}")
        except OSError as exc:
            record("11-opencode-json", False, f"cannot read: {exc}")
        else:
            has_schema = "$schema" in cfg
            default_ok = cfg.get("default_agent") == "team-lead"
            record(
                "11-opencode-json",
                has_schema and default_ok,
                f"$schema={'present' if has_schema else 'MISSING'} "
                f"default_agent={cfg.get('default_agent')!r}",
            )

    # ---- 12: AGENTS.md exists and has <=140 lines (and is not empty)
    if not AGENTS_MD_PATH.is_file():
        record("12-agents-md", False, "AGENTS.md missing")
    else:
        try:
            lines = AGENTS_MD_PATH.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:
            record("12-agents-md", False, f"cannot read: {exc}")
        else:
            count = len(lines)
            record("12-agents-md", 0 < count <= 140, f"lines={count} (required: 1..140)")

    # ---- 13: exactly 28 living docs (14 STATUS.md + 14 JOURNAL.md)
    status_dirs = sorted(p.parent.name for p in TEAM_AGENTS_DIR.glob("*/STATUS.md"))
    journal_dirs = sorted(p.parent.name for p in TEAM_AGENTS_DIR.glob("*/JOURNAL.md"))
    missing_status = sorted(set(AGENT_NAMES) - set(status_dirs))
    missing_journal = sorted(set(AGENT_NAMES) - set(journal_dirs))
    extra_status = sorted(set(status_dirs) - set(AGENT_NAMES))
    extra_journal = sorted(set(journal_dirs) - set(AGENT_NAMES))
    ok_docs = (
        len(status_dirs) == 14
        and len(journal_dirs) == 14
        and not missing_status
        and not missing_journal
        and not extra_status
        and not extra_journal
    )
    record(
        "13-living-docs-28",
        ok_docs,
        f"STATUS={len(status_dirs)}/14 missing={_names(missing_status)} "
        f"extra={_names(extra_status)} | JOURNAL={len(journal_dirs)}/14 "
        f"missing={_names(missing_journal)} extra={_names(extra_journal)}",
    )

    # ---- 14: every .opencode/team/... reference in prompts exists on disk
    refs: set[str] = set()
    for text in text_map.values():
        for match in TEAM_PATH_RE.findall(text):
            if "<" in match:  # template placeholder, not a real path
                continue
            refs.add(match.rstrip("/."))
    missing_refs = sorted(ref for ref in refs if not (ROOT / ref).exists())
    record(
        "14-team-path-crossrefs",
        bool(refs) and not missing_refs,
        f"refs={len(refs)} missing={_names(missing_refs)}",
    )

    # ---- 15: no case-insensitive collision with built-in agent names
    builtins_norm = {normalize_name(name) for name in BUILTIN_AGENT_NAMES}
    collisions = sorted(
        name for name in AGENT_NAMES if normalize_name(name) in builtins_norm
    )
    record(
        "15-builtin-name-collision",
        not collisions,
        f"collisions={_names(collisions)}",
    )

    # ---- report
    for check_id, ok, detail in results:
        print(f"{'PASS' if ok else 'FAIL'}  {check_id}  |  {detail}")
    passed = sum(1 for _, ok, _ in results if ok)
    total = len(results)
    print("-" * 72)
    print(f"SUMMARY: {passed}/{total} checks passed")
    print(f"RESULT: {'PASS' if passed == total else 'FAIL'}")
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
