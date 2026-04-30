DEFAULT_SCOPES = [
    "offline_access",
    "User.Read",
    "Mail.Read",
    "Mail.ReadWrite",
    "Mail.Send",
]


def joined() -> str:
    return " ".join(DEFAULT_SCOPES)
