from __future__ import annotations

import json
from enum import Enum
from typing import Callable, Iterable, TextIO


class Format(str, Enum):
    JSON = "json"
    TEXT = "text"
    TABLE = "table"


def render_rows(w: TextIO, fmt: Format, rows: list[dict], text_line: Callable[[dict], str]) -> None:
    if fmt is Format.JSON:
        json.dump(rows, w, indent=2)
        w.write("\n")
    elif fmt is Format.TEXT:
        for row in rows:
            w.write(text_line(row) + "\n")
    else:
        _render_table(w, rows)


def _render_table(w: TextIO, rows: Iterable[dict]) -> None:
    rows = list(rows)
    headers = ["ID", "From", "Subject", "Received", "Unread"]
    keys = ["id", "from", "subject", "received", "unread"]
    values = [[str(row.get(k, "")) for k in keys] for row in rows]
    widths = [len(h) for h in headers]
    for value_row in values:
        widths = [max(width, len(value)) for width, value in zip(widths, value_row)]

    def line(left: str, fill: str, sep: str, right: str) -> str:
        return left + sep.join(fill * (width + 2) for width in widths) + right

    def row(cells: list[str]) -> str:
        return "│" + "│".join(f" {cell:<{width}} " for cell, width in zip(cells, widths)) + "│"

    w.write(line("╭", "─", "┬", "╮") + "\n")
    w.write(row(headers) + "\n")
    w.write(line("├", "─", "┼", "┤") + "\n")
    for value_row in values:
        w.write(row(value_row) + "\n")
    w.write(line("╰", "─", "┴", "╯") + "\n")
