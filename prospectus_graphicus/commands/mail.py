from __future__ import annotations

import sys
from typing import TextIO

import requests

from prospectus_graphicus.auctoritas import KeyringStore, TokenStore, acquire_access_token
from prospectus_graphicus.config import Config
from prospectus_graphicus.graphus import GraphClient
from prospectus_graphicus.output import Format, render_rows


def _message_to_epistula(message: dict) -> dict:
    email = (message.get("from") or {}).get("emailAddress") or {}
    address = email.get("address") or ""
    name = email.get("name") or ""
    sender = f"{name} <{address}>" if name and address else address
    return {
        "id": message.get("id", ""),
        "from": sender,
        "subject": message.get("subject", ""),
        "received": message.get("receivedDateTime", ""),
        "unread": not bool(message.get("isRead", False)),
    }


def run_list(
    config: Config,
    session: requests.Session,
    beta: bool,
    top: int,
    fmt: Format,
    stdout: TextIO = sys.stdout,
) -> None:
    run_list_with_store(config, session, KeyringStore(), beta, top, fmt, stdout)


def run_list_with_store(
    config: Config,
    session: requests.Session,
    store: TokenStore,
    beta: bool,
    top: int,
    fmt: Format,
    stdout: TextIO = sys.stdout,
) -> None:
    token = acquire_access_token(session, config, store)
    client = GraphClient(session, config, beta, token)
    page = client.get_json(
        "/me/messages",
        {
            "$top": str(top),
            "$select": "id,subject,from,receivedDateTime,isRead",
            "$orderby": "receivedDateTime desc",
        },
    )
    rows = [_message_to_epistula(m) for m in page.get("value", [])]
    render_rows(
        stdout,
        fmt,
        rows,
        lambda e: f"{e['received']}  {e['from'] or '-'}  {e['subject']}",
    )
