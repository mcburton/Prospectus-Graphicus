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

1. Register an Azure app — see [`docs/azure-app-registration.md`](docs/azure-app-registration.md).
2. Create `~/.config/prospectus/config.toml`:

   ```toml
   [auth]
   client_id = "00000000-0000-0000-0000-000000000000"
   tenant_id = "pitt.edu"
   ```

3. Sign in:

   ```sh
   prospectus auth login
   ```

4. List mail:

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
