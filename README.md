# Prospectus Graphicus

> *Prospectus Graphicus: Instrumentum Lineae Iussorum ad Epistulas Prospecti per Graphum Tractandas.*
>
> "Prospectus Graphicus: A command-line instrument for handling Outlook's letters through the Graph."

A composable, Unix-philosophy CLI for the Microsoft Graph API, starting with
Outlook mail and designed to expand to calendar, contacts, and beyond. Each
invocation is a self-contained Graph call — no daemon, no background sync.

The binary is named `prospectus`. Output is JSON by default so it pipes cleanly
into `jq`.

## Status

Early. Exactly two commands are implemented end-to-end:

- `prospectus auth login` — OAuth2 device code flow; stores a refresh token in
  the OS keyring.
- `prospectus mail list [--top N]` — lists messages from `/me/messages`.

More slices (`mail send`, `mail get`, `calendar events list`, …) to follow.

## Design principles

- **Composable and pipe-friendly.** JSON default; `--output text|table` for
  humans.
- **Stateless.** Each invocation authenticates and exits. No daemon.
- **Honest errors.** Graph error `code` and `requestId` are surfaced verbatim.
- **Latin internals.** Commands stay English (`mail list`). Internal modules
  and types use Latin in the style of old scholarly books:
  - `auctoritas` — authority (auth)
  - `graphus` — the Graph (HTTP client)
  - `epistula` — a mail message
  - `scrinium`, `nuntius`, `kalendarium`, `liber` — reserved for later
    (folders, senders, calendars, contacts)

## Install

Requires a recent stable Rust (pinned in `rust-toolchain.toml`).

```sh
just install          # cargo install --path . --locked
```

## Configure

The default config uses Microsoft's **Graph PowerShell** first-party public
client ID, so you can sign in without registering your own Azure app.

1. Create `~/.config/prospectus/config.toml`:

   ```toml
   [auth]
   # Microsoft Graph PowerShell first-party public client ID.
   client_id = "14d82eec-204b-4c2f-b7e8-296a70dab67e"
   tenant_id = "pitt.edu"
   ```

2. Sign in:

   ```sh
   prospectus auth login
   ```

   The sign-in screen is branded "Microsoft Graph PowerShell" — that's
   expected. See [`docs/azure-app-registration.md`](docs/azure-app-registration.md)
   for tradeoffs and how to swap in your own Azure app registration later.

3. List mail:

   ```sh
   prospectus mail list --top 10
   prospectus mail list --output table
   prospectus mail list | jq '.[] | {subject, from}'
   ```

## Development

```sh
just            # list tasks
just build
just test
just fmt
just clippy
just run -- mail list --top 5
```

## License

MIT. See [`LICENSE`](LICENSE).
