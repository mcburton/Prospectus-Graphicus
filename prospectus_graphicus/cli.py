from __future__ import annotations

import argparse
import sys
from pathlib import Path

from .config import Config
from .errors import ProspectusError
from .output import Format

USER_AGENT = "prospectus-graphicus/0.1.0"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="prospectus",
        description="Prospectus Graphicus — a command-line instrument for handling Outlook's letters through the Microsoft Graph.",
    )
    parser.add_argument("--beta", action="store_true", help="Use the Graph beta endpoint instead of v1.0.")
    parser.add_argument("--output", choices=[f.value for f in Format], default=Format.JSON.value)
    parser.add_argument("--config", type=Path, default=None, help="Override the config path.")

    sub = parser.add_subparsers(dest="command", required=True)

    auth_p = sub.add_parser("auth", help="Authentication — sign in and manage credentials.")
    auth_sub = auth_p.add_subparsers(dest="auth_command", required=True)
    auth_sub.add_parser("login", help="Sign in via OAuth2 device code flow.")

    mail_p = sub.add_parser("mail", help="Mail — epistulae.")
    mail_sub = mail_p.add_subparsers(dest="mail_command", required=True)
    list_p = mail_sub.add_parser("list", help="List messages from the signed-in user's inbox.")
    list_p.add_argument("--top", type=int, default=25, help="Maximum number of messages to return.")

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        import requests
        from .commands import auth, mail

        cfg = Config.load(args.config)
        session = requests.Session()
        session.headers.update({"User-Agent": USER_AGENT})

        if args.command == "auth" and args.auth_command == "login":
            auth.run(cfg, session)
        elif args.command == "mail" and args.mail_command == "list":
            mail.run_list(cfg, session, args.beta, args.top, Format(args.output))
        else:
            parser.error("unknown command")
        return 0
    except ModuleNotFoundError as exc:
        print(f"missing Python dependency: {exc.name}; run `uv sync`", file=sys.stderr)
        return 1
    except ProspectusError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
