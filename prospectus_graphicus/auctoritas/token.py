from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Protocol

import keyring
import requests

from prospectus_graphicus.config import Config
from prospectus_graphicus.errors import NotAuthenticated, OAuth2Error
from . import scopes

KEYRING_SERVICE = "prospectus-graphicus"
KEYRING_ACCOUNT = "default"


@dataclass(frozen=True)
class TokenSet:
    access_token: str
    refresh_token: str | None
    expires_at: datetime
    scope: str | None
    token_type: str

    @classmethod
    def from_oauth(cls, body: dict) -> "TokenSet":
        return cls(
            access_token=body["access_token"],
            refresh_token=body.get("refresh_token"),
            expires_at=datetime.now(timezone.utc) + timedelta(seconds=int(body.get("expires_in", 0))),
            scope=body.get("scope"),
            token_type=body.get("token_type", "Bearer"),
        )

    def is_expired(self) -> bool:
        return datetime.now(timezone.utc) + timedelta(seconds=60) >= self.expires_at


class TokenStore(Protocol):
    def save_refresh_token(self, token: str) -> None: ...
    def load_refresh_token(self) -> str | None: ...
    def clear(self) -> None: ...


class KeyringStore:
    def save_refresh_token(self, token: str) -> None:
        keyring.set_password(KEYRING_SERVICE, KEYRING_ACCOUNT, token)

    def load_refresh_token(self) -> str | None:
        return keyring.get_password(KEYRING_SERVICE, KEYRING_ACCOUNT)

    def clear(self) -> None:
        try:
            keyring.delete_password(KEYRING_SERVICE, KEYRING_ACCOUNT)
        except keyring.errors.PasswordDeleteError:
            pass


def _token_url(cfg: Config) -> str:
    return f"{cfg.login_base().rstrip('/')}/{cfg.auth.tenant_id}/oauth2/v2.0/token"


def refresh(session: requests.Session, cfg: Config, refresh_token: str) -> TokenSet:
    resp = session.post(
        _token_url(cfg),
        data={
            "client_id": cfg.auth.client_id,
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "scope": scopes.joined(),
        },
    )
    body = _json_or_empty(resp)
    if not resp.ok:
        raise OAuth2Error(body.get("error", "unknown"), body.get("error_description", ""))
    return TokenSet.from_oauth(body)


def acquire_access_token(session: requests.Session, cfg: Config, store: TokenStore) -> TokenSet:
    rt = store.load_refresh_token()
    if not rt:
        raise NotAuthenticated()
    ts = refresh(session, cfg, rt)
    if ts.refresh_token:
        store.save_refresh_token(ts.refresh_token)
    return ts


def _json_or_empty(resp: requests.Response) -> dict:
    try:
        body = resp.json()
        return body if isinstance(body, dict) else {}
    except ValueError:
        return {}
