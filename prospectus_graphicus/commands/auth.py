from __future__ import annotations

import sys
from typing import TextIO

import requests

from prospectus_graphicus.auctoritas import KeyringStore, TokenStore
from prospectus_graphicus.auctoritas.device_code import DeviceCodeResponse, login_interactive
from prospectus_graphicus.config import Config


def run(config: Config, session: requests.Session, stderr: TextIO = sys.stderr) -> None:
    run_with_store(config, session, KeyringStore(), stderr)


def run_with_store(config: Config, session: requests.Session, store: TokenStore, stderr: TextIO = sys.stderr) -> None:
    def on_code(device: DeviceCodeResponse) -> None:
        if device.message:
            print(device.message, file=stderr)
        else:
            print(f"To sign in, visit {device.verification_uri} and enter code {device.user_code}", file=stderr)
        print("Waiting for authorization...", file=stderr)

    login_interactive(session, config, store, on_code)
    print("Login successful. Refresh token stored in system keyring.", file=stderr)
