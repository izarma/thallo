#!/usr/bin/env python3
"""Convert image-to-ascii crate output into thallo's AsciiAnimation JSON format.

The image-to-ascii crate (https://crates.io/crates/image-to-ascii) emits a JSON
array of frame strings. This game expects an object with metadata, animation
config and a frames array where each frame has a contentString field.

Example:
    python3 tools/convert_image_to_ascii_json.py assets/my_anim.json assets/my_anim_game.json --fps 24

You can also overwrite the input file in-place:
    python3 tools/convert_image_to_ascii_json.py assets/my_anim.json assets/my_anim.json --fps 24
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def convert(
    frames: list[str],
    title: str,
    fps: int,
    background_color: str = "#000000",
) -> dict[str, Any]:
    if not frames:
        raise ValueError("input JSON contains no frames")

    # Measure canvas size from the first frame. Each frame is expected to be a
    # single string with embedded newline characters.
    first_frame = frames[0]
    lines = first_frame.split("\n")
    width = max(len(line) for line in lines) if lines else 0
    height = len(lines)

    return {
        "metadata": {
            "exportedAt": None,
            "exportVersion": "1.0.0",
            "appVersion": "2.1.11",
            "description": "ASCII Motion Animation - Human Readable Format",
            "title": title,
            "frameCount": len(frames),
            "canvasSize": {"width": width, "height": height},
        },
        "canvas": {
            "width": width,
            "height": height,
            "backgroundColor": background_color,
        },
        "typography": {
            "fontSize": 29,
            "characterSpacing": 1,
            "lineSpacing": 1,
        },
        "animation": {
            "frameRate": fps,
            "looping": True,
            "currentFrame": 0,
        },
        "frames": [
            {
                "title": f"Frame {i}",
                "duration": 1000.0 / fps,
                "contentString": frame,
            }
            for i, frame in enumerate(frames)
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Convert image-to-ascii JSON into thallo's AsciiAnimation format."
    )
    parser.add_argument("input", type=Path, help="image-to-ascii JSON file")
    parser.add_argument("output", type=Path, help="output AsciiAnimation JSON file")
    parser.add_argument(
        "--fps",
        type=int,
        default=24,
        help="frames per second for the animation (default: 24)",
    )
    parser.add_argument(
        "--title",
        type=str,
        default=None,
        help="animation title (default: derived from input filename)",
    )
    parser.add_argument(
        "--bg",
        type=str,
        default="#000000",
        help="background color hex value (default: #000000)",
    )
    parser.add_argument(
        "--indent",
        type=int,
        default=None,
        help="indentation for output JSON (default: compact)",
    )

    args = parser.parse_args()

    title = args.title or args.input.stem

    with args.input.open("r", encoding="utf-8") as f:
        raw = json.load(f)

    if not isinstance(raw, list):
        print(
            f"error: expected input JSON to be a list of frame strings, got {type(raw).__name__}",
            file=sys.stderr,
        )
        return 1

    try:
        converted = convert(raw, title, args.fps, args.bg)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8") as f:
        json.dump(converted, f, indent=args.indent)

    print(
        f"wrote {converted['metadata']['frameCount']} frames "
        f"({converted['canvas']['width']}x{converted['canvas']['height']}) "
        f"at {args.fps} fps to {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
