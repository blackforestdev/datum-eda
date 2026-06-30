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


def parse_doc_tokens() -> dict[str, str]:
    text = DESIGN_BOOK.read_text()
    tokens: dict[str, str] = {}
    for token, hex_value in re.findall(r"`((?:color|content)\.[^`]+)`\s*\|\s*`(#[0-9A-Fa-f]{6})`", text):
        tokens[token] = hex_value.upper()
    return tokens


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
    rust_tokens = parse_rust_tokens()
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
        f"{len(TEXT_TOKENS) * len(SURFACE_TOKENS)} contrast checks)."
    )


if __name__ == "__main__":
    main()
