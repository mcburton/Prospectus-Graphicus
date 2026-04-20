# Azure app registration

Prospectus Graphicus is a **public client** (no client secret). It authenticates
with the OAuth2 device authorization grant (RFC 8628) against the Microsoft
identity platform.

You need two values for `~/.config/prospectus/config.toml`:

- `client_id` — the Azure app registration's **Application (client) ID**
- `tenant_id` — your AAD tenant ID, or a domain like `pitt.edu`

## Option 1: Register the app in your own tenant

This is the cleanest path if your tenant admin allows self-service app
registrations.

1. Sign in to <https://portal.azure.com> with your organizational account
   (e.g. your Pitt credentials).
2. Navigate to **Microsoft Entra ID → App registrations → New registration**.
3. Fill in:
   - **Name**: `Prospectus Graphicus`
   - **Supported account types**: *Accounts in this organizational directory
     only* (single tenant).
   - **Redirect URI**: leave blank.
4. Click **Register**.
5. On the app's **Overview** page, copy:
   - *Application (client) ID* → `client_id`
   - *Directory (tenant) ID* → `tenant_id` (or use the domain, e.g. `pitt.edu`)
6. Go to **Authentication → Advanced settings** and set
   **Allow public client flows** to **Yes**. Save. This is required for the
   device code flow.
7. Go to **API permissions → Add a permission → Microsoft Graph →
   Delegated permissions** and add:
   - `offline_access`
   - `User.Read`
   - `Mail.Read`
   - `Mail.ReadWrite`
   - `Mail.Send`

   These are granted at the user level on first sign-in. Your tenant may
   require admin consent depending on policy — in that case, click
   **Grant admin consent for <your tenant>** if you have the role, or ask
   an admin to grant it.

## Option 2: Can't register in your tenant (common at universities)

Many organizations — including most universities — restrict app registrations
to administrators. If the **New registration** button is greyed out or the
save fails, you have two choices:

### 2a. Ask IT

Contact your tenant admin (at Pitt, that's CSSD) and request an app
registration for a personal CLI. Mention:

- It's a **public client** (no secret).
- It uses the **OAuth2 device code flow**.
- It requires the delegated Graph permissions listed above.
- There is no redirect URI.
- It only acts on behalf of the signed-in user.

### 2b. Use a personal tenant

Sign up for a free Microsoft 365 developer tenant (or use any tenant you
control) and register a **multi-tenant** app there. Then set `tenant_id =
"common"` in your config to authenticate against any organizational account.
The downside: your sign-ins route through an app you don't own at the
organization level, which is fine technically but defeats the single-tenant
preference.

## Troubleshooting

- **`AADSTS7000218` / "client_assertion or client_secret required"** — *Allow
  public client flows* is not enabled. Fix in the app's **Authentication**
  blade.
- **`AADSTS65001` / "consent required"** — your tenant requires admin consent
  for one of the requested scopes. Ask an admin to grant it, or request the
  scopes individually.
- **`AADSTS50020` / "user does not exist in tenant"** — your `tenant_id` in
  `config.toml` points at a tenant the signed-in user isn't a member of.
  Double-check the tenant ID or use `common`.
