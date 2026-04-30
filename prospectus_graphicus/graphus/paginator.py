from __future__ import annotations

from typing import Any

from . import GraphClient


def collect_all(client: GraphClient, first_path: str) -> list[Any]:
    out: list[Any] = []
    page = client.get_json(first_path)
    out.extend(page.get("value", []))
    while page.get("@odata.nextLink"):
        page = client.get_url(page["@odata.nextLink"])
        out.extend(page.get("value", []))
    return out
