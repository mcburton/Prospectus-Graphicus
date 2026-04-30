from __future__ import annotations

import os
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

from .errors import ConfigError

GRAPH_V1 = "https://graph.microsoft.com/v1.0"
GRAPH_BETA = "https://graph.microsoft.com/beta"
LOGIN_BASE = "https://login.microsoftonline.com"

CONFIG_TEMPLATE = '''# Prospectus Graphicus configuration.
# See docs/azure-app-registration.md for details.

[auth]
# Microsoft Graph PowerShell first-party public client ID.
# Replace with your own Azure app registration when you have one.
client_id = "14d82eec-204b-4c2f-b7e8-296a70dab67e"
tenant_id = "pitt.edu"
'''


@dataclass(frozen=True)
class AuthConfig:
    client_id: str
    tenant_id: str


@dataclass(frozen=True)
class GraphConfig:
    endpoint_override: str | None = None
    login_override: str | None = None


@dataclass(frozen=True)
class Config:
    auth: AuthConfig
    graph: GraphConfig = field(default_factory=GraphConfig)

    @classmethod
    def load(cls, path: Path | None = None) -> "Config":
        return cls.load_from(path or default_config_path())

    @classmethod
    def load_from(cls, path: Path) -> "Config":
        if not path.exists():
            raise ConfigError(
                f"config file not found at {path}. Create it with your Azure app registration "
                "details — see docs/azure-app-registration.md"
            )
        raw = tomllib.loads(path.read_text())
        try:
            auth = raw["auth"]
            auth_cfg = AuthConfig(client_id=auth["client_id"], tenant_id=auth["tenant_id"])
        except KeyError as exc:
            raise ConfigError(f"missing config key: {exc}") from exc
        graph = raw.get("graph", {})
        return cls(
            auth=auth_cfg,
            graph=GraphConfig(
                endpoint_override=graph.get("endpoint_override"),
                login_override=graph.get("login_override"),
            ),
        )

    def graph_base(self, beta: bool) -> str:
        if self.graph.endpoint_override:
            return self.graph.endpoint_override
        return GRAPH_BETA if beta else GRAPH_V1

    def login_base(self) -> str:
        return self.graph.login_override or LOGIN_BASE


def default_config_path() -> Path:
    base = os.environ.get("XDG_CONFIG_HOME")
    if base:
        return Path(base) / "prospectus" / "config.toml"
    return Path.home() / ".config" / "prospectus" / "config.toml"
