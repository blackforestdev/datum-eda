#!/usr/bin/env python3
"""Gate Datum GUI Design Book token mirroring and chrome contrast."""

from __future__ import annotations

import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DESIGN_BOOK = ROOT / "docs/gui/VISUAL_LANGUAGE.md"
RUST_TOKENS = ROOT / "crates/gui-render/src/design_tokens.rs"
BOARD_EDITOR_PROTOTYPE = ROOT / "docs/gui/prototypes/board-editor.html"

REQUIRED_DOC_TOKENS = {
    "color.canvas": "CANVAS",
    "color.bg.base": "BG_BASE",
    "color.surface.01": "SURFACE_01",
    "color.surface.02": "SURFACE_02",
    "color.surface.03": "SURFACE_03",
    "color.border.subtle": "BORDER_SUBTLE",
    "color.border.strong": "BORDER_STRONG",
    "color.text.primary": "TEXT_PRIMARY",
    "color.text.secondary": "TEXT_SECONDARY",
    "color.text.muted": "TEXT_MUTED",
    "color.text.onAccent": "TEXT_ON_ACCENT",
    "color.accent": "ACCENT",
    "color.accent.hover": "ACCENT_HOVER",
    "color.accent.pressed": "ACCENT_PRESSED",
    "color.accent.tint": "ACCENT_TINT",
    "color.status.error": "STATUS_ERROR",
    "color.status.warn": "STATUS_WARN",
    "color.status.success": "STATUS_SUCCESS",
    "color.status.info": "STATUS_INFO",
}

REQUIRED_CONTENT_TOKENS = {
    "content.copper.front": "COPPER_FRONT",
    "content.copper.back": "COPPER_BACK",
    "content.copper.in1": "COPPER_IN1",
    "content.copper.in2": "COPPER_IN2",
    "content.silk.top": "SILK_TOP",
    "content.silk.bottom": "SILK_BOTTOM",
    "content.mask": "MASK",
    "content.paste": "PASTE",
    "content.edge": "EDGE",
    "content.pad": "PAD",
    "content.via": "VIA",
    "content.ratsnest": "RATSNEST",
    "content.drc.error": "DRC_ERROR",
    "content.drc.warn": "DRC_WARN",
    "content.exclusion": "EXCLUSION",
}

TEXT_TOKENS = [
    ("color.text.primary", 4.5),
    ("color.text.secondary", 4.5),
    # The Design Book explicitly limits muted text to metadata/captions.
    ("color.text.muted", 3.0),
]

SURFACE_TOKENS = [
    "color.bg.base",
    "color.surface.01",
    "color.surface.02",
    "color.surface.03",
]

REQUIRED_TYPE_TOKENS = {
    "type.display": ("DISPLAY", "IBM Plex Sans", 16.0, 600, 22.0),
    "type.header": ("HEADER", "IBM Plex Sans", 12.0, 600, 16.0),
    "type.body": ("BODY", "IBM Plex Sans", 13.0, 400, 18.0),
    "type.strong": ("STRONG", "IBM Plex Sans", 13.0, 500, 18.0),
    "type.data": ("DATA", "IBM Plex Mono", 12.0, 400, 16.0),
    "type.caption": ("CAPTION", "IBM Plex Sans", 11.0, 400, 14.0),
    "type.micro": ("MICRO", "IBM Plex Sans", 10.0, 500, 12.0),
}

REQUIRED_SPACING_TOKENS = {
    "sp.01": "SP_01",
    "sp.02": "SP_02",
    "sp.03": "SP_03",
    "sp.04": "SP_04",
    "sp.05": "SP_05",
    "sp.06": "SP_06",
    "sp.07": "SP_07",
    "sp.08": "SP_08",
    "sp.09": "SP_09",
    "sp.10": "SP_10",
    "sp.11": "SP_11",
    "sp.12": "SP_12",
    "sp.13": "SP_13",
}

REQUIRED_RADIUS_TOKENS = {
    "radius.sm": "SM",
    "radius.md": "MD",
    "radius.lg": "LG",
}

PROTOTYPE_ROOT_TOKEN_MAP = {
    "--canvas": "color.canvas",
    "--bg": "color.bg.base",
    "--s01": "color.surface.01",
    "--s02": "color.surface.02",
    "--s03": "color.surface.03",
    "--bd-sub": "color.border.subtle",
    "--bd-str": "color.border.strong",
    "--tx": "color.text.primary",
    "--tx2": "color.text.secondary",
    "--tx3": "color.text.muted",
    "--onAccent": "color.text.onAccent",
    "--acc": "color.accent",
    "--acc-h": "color.accent.hover",
    "--acc-p": "color.accent.pressed",
    "--acc-tint": "color.accent.tint",
    "--err": "color.status.error",
    "--warn": "color.status.warn",
    "--ok": "color.status.success",
    "--info": "color.status.info",
    "--cu-f": "content.copper.front",
    "--cu-b": "content.copper.back",
    "--cu-i1": "content.copper.in1",
    "--cu-i2": "content.copper.in2",
    "--silk": "content.silk.top",
    "--mask": "content.mask",
    "--pad": "content.pad",
    "--via": "content.via",
    "--rat": "content.ratsnest",
    "--edge": "content.edge",
}


def parse_doc_tokens() -> dict[str, str]:
    text = DESIGN_BOOK.read_text()
    tokens: dict[str, str] = {}
    for token, hex_value in re.findall(r"`((?:color|content)\.[^`]+)`\s*\|\s*`(#[0-9A-Fa-f]{6})`", text):
        tokens[token] = hex_value.upper()
    return tokens


def parse_doc_type_tokens() -> dict[str, tuple[str, float, int, float]]:
    text = DESIGN_BOOK.read_text()
    values: dict[str, tuple[str, float, int, float]] = {}
    for token, family, size, weight, line, _use in re.findall(
        r"\|\s*`(type\.[^`]+)`\s*\|\s*([^|]+?)\s*\|\s*([0-9.]+)\s*\|\s*([0-9]+)\s*\|\s*([0-9.]+)\s*\|\s*([^|]+)\|",
        text,
    ):
        values[token] = (family.strip(), float(size), int(weight), float(line))
    return values


def parse_doc_numeric_tokens(prefix: str) -> dict[str, float]:
    text = DESIGN_BOOK.read_text()
    values: dict[str, float] = {}
    for token, value in re.findall(rf"`({re.escape(prefix)}\.[^`]+)`\s*\|\s*([0-9.]+)", text):
        values[token] = float(value)
    for token, value in re.findall(rf"`({re.escape(prefix)}\.[^`]+)`\s+([0-9.]+)px", text):
        values[token] = float(value)
    return values


def parse_prototype_root_tokens() -> dict[str, str]:
    text = BOARD_EDITOR_PROTOTYPE.read_text()
    root = re.search(r":root\s*\{(?P<body>.*?)\}", text, flags=re.S)
    if not root:
        fail("board-editor prototype has no :root token block")
    values: dict[str, str] = {}
    for name, hex_value in re.findall(
        r"(--[A-Za-z0-9_-]+)\s*:\s*(#[0-9A-Fa-f]{6})",
        root.group("body"),
    ):
        values[name] = hex_value.upper()
    return values


def parse_rust_tokens() -> dict[str, str]:
    text = RUST_TOKENS.read_text()
    values: dict[str, str] = {}
    for name, r, g, b in re.findall(
        r"pub\(crate\) const ([A-Z0-9_]+): Rgb = srgb\(0x([0-9A-Fa-f]{2}), 0x([0-9A-Fa-f]{2}), 0x([0-9A-Fa-f]{2})\);",
        text,
    ):
        values[name] = f"#{r}{g}{b}".upper()
    alias_match = re.search(r"pub\(crate\) const SELECTION: Rgb = chrome::ACCENT;", text)
    if alias_match and "ACCENT" in values:
        values["SELECTION"] = values["ACCENT"]
    for name, target in re.findall(r"pub\(crate\) const ([A-Z0-9_]+): Rgb = chrome::([A-Z0-9_]+);", text):
        if target in values:
            values[name] = values[target]
    return values


def parse_rust_numeric_constants(module: str) -> dict[str, float]:
    text = RUST_TOKENS.read_text()
    body = re.search(rf"pub\(crate\) mod {module} \{{(?P<body>.*?)\n\}}", text, flags=re.S)
    if not body:
        fail(f"Rust token mirror is missing module {module}")
    values: dict[str, float] = {}
    for name, value in re.findall(
        r"pub\(crate\) const ([A-Z0-9_]+): (?:f32|u16) = ([0-9.]+);",
        body.group("body"),
    ):
        values[name] = float(value)
    return values


def srgb_to_linear(channel: float) -> float:
    value = channel / 255.0
    if value <= 0.03928:
        return value / 12.92
    return ((value + 0.055) / 1.055) ** 2.4


def relative_luminance(hex_value: str) -> float:
    r = int(hex_value[1:3], 16)
    g = int(hex_value[3:5], 16)
    b = int(hex_value[5:7], 16)
    return (
        0.2126 * srgb_to_linear(r)
        + 0.7152 * srgb_to_linear(g)
        + 0.0722 * srgb_to_linear(b)
    )


def wcag_contrast(foreground: str, background: str) -> float:
    fg = relative_luminance(foreground)
    bg = relative_luminance(background)
    lighter = max(fg, bg)
    darker = min(fg, bg)
    return (lighter + 0.05) / (darker + 0.05)


def apca_lc_approx(foreground: str, background: str) -> float:
    # Lightweight gate approximation: enough to catch directionally bad dense
    # text regressions without adding an external APCA dependency.
    fg = relative_luminance(foreground)
    bg = relative_luminance(background)
    polarity = 1.14 if bg > fg else 1.14
    return abs((bg**0.56 - fg**0.57) * 100.0 * polarity)


def fail(message: str) -> None:
    print(f"GUI design token gate failed: {message}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    doc_tokens = parse_doc_tokens()
    doc_type_tokens = parse_doc_type_tokens()
    doc_spacing_tokens = parse_doc_numeric_tokens("sp")
    doc_radius_tokens = parse_doc_numeric_tokens("radius")
    prototype_tokens = parse_prototype_root_tokens()
    rust_tokens = parse_rust_tokens()
    rust_typography = parse_rust_numeric_constants("typography")
    rust_spacing = parse_rust_numeric_constants("spacing")
    rust_radius = parse_rust_numeric_constants("radius")
    token_map = REQUIRED_DOC_TOKENS | REQUIRED_CONTENT_TOKENS | {
        "content.selection": "SELECTION"
    }

    missing_doc = sorted(token for token in token_map if token not in doc_tokens)
    if missing_doc:
        fail(f"Design Book is missing tokens: {', '.join(missing_doc)}")

    missing_rust = sorted(name for name in token_map.values() if name not in rust_tokens)
    if missing_rust:
        fail(f"Rust token mirror is missing constants: {', '.join(missing_rust)}")

    mismatches = []
    for doc_name, rust_name in token_map.items():
        if doc_tokens[doc_name] != rust_tokens[rust_name]:
            mismatches.append(f"{doc_name}: docs {doc_tokens[doc_name]} != Rust {rust_tokens[rust_name]}")
    if mismatches:
        fail("; ".join(mismatches))

    prototype_mismatches = []
    for prototype_name, doc_name in PROTOTYPE_ROOT_TOKEN_MAP.items():
        if prototype_name not in prototype_tokens:
            prototype_mismatches.append(f"prototype missing {prototype_name}")
        elif prototype_tokens[prototype_name] != doc_tokens[doc_name]:
            prototype_mismatches.append(
                f"{prototype_name}: prototype {prototype_tokens[prototype_name]} != {doc_name} {doc_tokens[doc_name]}"
            )
    if prototype_mismatches:
        fail("; ".join(prototype_mismatches))

    type_failures = []
    for token, (rust_prefix, expected_family, expected_size, expected_weight, expected_line) in REQUIRED_TYPE_TOKENS.items():
        if token not in doc_type_tokens:
            type_failures.append(f"Design Book missing {token}")
            continue
        family, size, weight, line = doc_type_tokens[token]
        if family != expected_family:
            type_failures.append(f"{token}: family {family!r} != {expected_family!r}")
        if size != expected_size or weight != expected_weight or line != expected_line:
            type_failures.append(
                f"{token}: docs {(size, weight, line)} != expected {(expected_size, expected_weight, expected_line)}"
            )
        for suffix, expected in [
            ("SIZE", expected_size),
            ("WEIGHT", float(expected_weight)),
            ("LINE", expected_line),
        ]:
            rust_name = f"{rust_prefix}_{suffix}"
            actual = rust_typography.get(rust_name)
            if actual != expected:
                type_failures.append(f"typography::{rust_name}: Rust {actual} != docs {expected}")
    if type_failures:
        fail("; ".join(type_failures))

    spacing_failures = []
    for token, rust_name in REQUIRED_SPACING_TOKENS.items():
        actual = rust_spacing.get(rust_name)
        expected = doc_spacing_tokens.get(token)
        if actual != expected:
            spacing_failures.append(f"{token}/{rust_name}: docs {expected} != Rust {actual}")
    for token, rust_name in REQUIRED_RADIUS_TOKENS.items():
        actual = rust_radius.get(rust_name)
        expected = doc_radius_tokens.get(token)
        if actual != expected:
            spacing_failures.append(f"{token}/{rust_name}: docs {expected} != Rust {actual}")
    if spacing_failures:
        fail("; ".join(spacing_failures))

    # Consumption guard (VISUAL_LANGUAGE.md §7): board copper materials must be
    # built from the design_tokens seam, never from raw RGB array literals. The
    # token-driven path passes a named base color, so this only trips when a
    # `from_copper_material([0.xx, ...])` raw literal is reintroduced.
    render_src = (ROOT / "crates/gui-render/src/lib.rs").read_text()
    raw_copper = re.findall(r"from_copper_material\(\s*\[", render_src)
    if raw_copper:
        fail(
            f"{len(raw_copper)} copper material(s) built from raw RGB literals in "
            "gui-render/src/lib.rs; construct from design_tokens::content instead"
        )
    forbidden_literals = [
        ("board-field border", "[0.46, 0.49, 0.53]"),
        ("viewport status error", "[0.85, 0.40, 0.35]"),
        ("viewport status success", "[0.45, 0.72, 0.45]"),
        ("viewport status background", "[0.07, 0.08, 0.10]"),
    ]
    literal_failures = [label for label, literal in forbidden_literals if literal in render_src]
    if literal_failures:
        fail(
            "ad-hoc chrome RGB literals remain in gui-render/src/lib.rs: "
            + ", ".join(literal_failures)
        )

    contrast_failures = []
    for text_token, wcag_floor in TEXT_TOKENS:
        for surface_token in SURFACE_TOKENS:
            wcag = wcag_contrast(doc_tokens[text_token], doc_tokens[surface_token])
            apca = apca_lc_approx(doc_tokens[text_token], doc_tokens[surface_token])
            if wcag + 1e-9 < wcag_floor:
                contrast_failures.append(
                    f"{text_token} on {surface_token}: WCAG {wcag:.2f} < {wcag_floor:.1f}"
                )
            if text_token != "color.text.muted" and apca + 1e-9 < 60.0:
                contrast_failures.append(
                    f"{text_token} on {surface_token}: APCA approx Lc {apca:.1f} < 60.0"
                )
    if contrast_failures:
        fail("; ".join(contrast_failures))

    print(
        f"GUI design token gate passed ({len(token_map)} mirrored color tokens, "
        f"{len(REQUIRED_TYPE_TOKENS)} type tokens, "
        f"{len(REQUIRED_SPACING_TOKENS) + len(REQUIRED_RADIUS_TOKENS)} spacing/radius tokens, "
        f"{len(PROTOTYPE_ROOT_TOKEN_MAP)} prototype vars, "
        f"{len(TEXT_TOKENS) * len(SURFACE_TOKENS)} contrast checks, copper/chrome "
        "literal consumption verified)."
    )


if __name__ == "__main__":
    main()
