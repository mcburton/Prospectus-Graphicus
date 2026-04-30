from __future__ import annotations


class ProspectusError(Exception):
    """Base exception for Prospectus Graphicus."""


class ConfigError(ProspectusError):
    pass


class NotAuthenticated(ProspectusError):
    def __str__(self) -> str:
        return "no cached credentials found; run `prospectus auth login`"


class AuthorizationDeclined(ProspectusError):
    def __str__(self) -> str:
        return "authorization declined by user"


class DeviceCodeExpired(ProspectusError):
    def __str__(self) -> str:
        return "device code expired; please retry `prospectus auth login`"


class OAuth2Error(ProspectusError):
    def __init__(self, code: str, description: str = "") -> None:
        self.code = code
        self.description = description
        super().__init__(code, description)

    def __str__(self) -> str:
        return f"OAuth2 error: {self.code}: {self.description}"


class GraphError(ProspectusError):
    def __init__(self, status: int, code: str, message: str = "", request_id: str | None = None) -> None:
        self.status = status
        self.code = code
        self.message = message
        self.request_id = request_id
        super().__init__(status, code, message, request_id)

    def __str__(self) -> str:
        out = f"{self.status} {self.code}: {self.message}"
        if self.request_id:
            out += f" (requestId: {self.request_id})"
        return out
