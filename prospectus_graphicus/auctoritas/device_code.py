from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Callable

import requests

from prospectus_graphicus.config import Config
from prospectus_graphicus.errors import AuthorizationDeclined, DeviceCodeExpired, OAuth2Error
from . import scopes
from .token import TokenSet, TokenStore


@dataclass(frozen=True)
class DeviceCodeResponse:
    device_code: str
    user_code: str
    verification_uri: str
    expires_in: int
    interval: int
    message: str | None = None


def request_device_code(session: requests.Session, cfg: Config) -> DeviceCodeResponse:
    url = f"{cfg.login_base().rstrip('/')}/{cfg.auth.tenant_id}/oauth2/v2.0/devicecode"
    resp = session.post(url, data={"client_id": cfg.auth.client_id, "scope": scopes.joined()})
    body = _json_or_empty(resp)
    if not resp.ok:
        raise OAuth2Error(body.get("error", "unknown"), body.get("error_description", ""))
    return DeviceCodeResponse(
        device_code=body["device_code"],
        user_code=body["user_code"],
        verification_uri=body["verification_uri"],
        expires_in=int(body["expires_in"]),
        interval=int(body.get("interval", 5)),
        message=body.get("message"),
    )


def poll_for_token(session: requests.Session, cfg: Config, device: DeviceCodeResponse) -> TokenSet:
    url = f"{cfg.login_base().rstrip('/')}/{cfg.auth.tenant_id}/oauth2/v2.0/token"
    interval = max(device.interval, 1)
    deadline = time.monotonic() + max(device.expires_in, 60)

    while True:
        if time.monotonic() >= deadline:
            raise DeviceCodeExpired()
        time.sleep(interval)
        resp = session.post(
            url,
            data={
                "client_id": cfg.auth.client_id,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": device.device_code,
            },
        )
        body = _json_or_empty(resp)
        if resp.ok:
            return TokenSet.from_oauth(body)

        code = body.get("error", "unknown")
        if code == "authorization_pending":
            continue
        if code == "slow_down":
            interval += 5
            continue
        if code == "authorization_declined":
            raise AuthorizationDeclined()
        if code in {"expired_token", "code_expired"}:
            raise DeviceCodeExpired()
        raise OAuth2Error(code, body.get("error_description", ""))


def login_interactive(
    session: requests.Session,
    cfg: Config,
    store: TokenStore,
    on_code: Callable[[DeviceCodeResponse], None],
) -> TokenSet:
    device = request_device_code(session, cfg)
    on_code(device)
    token = poll_for_token(session, cfg, device)
    if token.refresh_token:
        store.save_refresh_token(token.refresh_token)
    return token


def _json_or_empty(resp: requests.Response) -> dict:
    try:
        body = resp.json()
        return body if isinstance(body, dict) else {}
    except ValueError:
        return {}
