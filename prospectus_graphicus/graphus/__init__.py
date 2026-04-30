from __future__ import annotations

from typing import Any

import requests

from prospectus_graphicus.auctoritas.token import TokenSet
from prospectus_graphicus.config import Config
from prospectus_graphicus.errors import GraphError


class GraphClient:
    def __init__(self, session: requests.Session, cfg: Config, beta: bool, token: TokenSet) -> None:
        self.session = session
        self.base = cfg.graph_base(beta).rstrip("/")
        self.access_token = token.access_token

    def get_json(self, path: str, params: dict[str, str] | list[tuple[str, str]] | None = None) -> Any:
        url = f"{self.base}/{path.lstrip('/')}"
        resp = self.session.get(url, headers={"Authorization": f"Bearer {self.access_token}"}, params=params)
        return _handle_response(resp)

    def get_url(self, url: str) -> Any:
        resp = self.session.get(url, headers={"Authorization": f"Bearer {self.access_token}"})
        return _handle_response(resp)


def _handle_response(resp: requests.Response) -> Any:
    if resp.ok:
        return resp.json()

    request_id = resp.headers.get("request-id") or resp.headers.get("client-request-id")
    try:
        body = resp.json()
    except ValueError:
        body = {}
    error = body.get("error", {}) if isinstance(body, dict) else {}
    inner = error.get("innerError", {}) if isinstance(error, dict) else {}
    raise GraphError(
        status=resp.status_code,
        code=error.get("code", "UnknownError"),
        message=error.get("message", ""),
        request_id=inner.get("request-id") or request_id,
    )
