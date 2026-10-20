"""Reject unusable native captures without treating pixel content as acceptance."""
from pathlib import Path

from PIL import Image


def require_native_capture_pixels(path: Path) -> None:
    """Keep the original PNG and fail on decode errors or wholly black RGB.

    Pillow is already used for repository PNG verification. This is an exact
    failure check, not a visibility threshold or evidence of correct rendering.
    """
    try:
        with Image.open(path) as image:
            if image.format != "PNG":
                raise ValueError("native capture is not PNG")
            image.verify()
        with Image.open(path) as image:
            image.load()
            extrema = image.convert("RGB").getextrema()
    except (OSError, ValueError, SyntaxError) as error:
        raise RuntimeError(f"Unusable native capture {path}: {error}") from error
    if all(low == high == 0 for low, high in extrema):
        raise RuntimeError(f"Unusable native capture {path}: wholly black RGB pixels; "
                           "original preserved, no retry or rendering acceptance")
