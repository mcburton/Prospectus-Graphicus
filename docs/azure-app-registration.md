# Authentication & Azure app registration

Prospectus Graphicus is a **public client** (no client secret). It signs in
using the OAuth2 device authorization grant (RFC 8628) against the Microsoft
identity platform, with your Pitt account.

You have three paths. In order of effort:

1. **Default (no registration required)** — use a Microsoft first-party
   client ID. Works out of the box for most users.
2. **Ask Pitt IT to consent** — if the default is blocked by your tenant's
   user-consent policy.
3. **Register your own app** — the officially correct long-term answer.

## Path 1: Default — Microsoft Graph PowerShell client ID

The shipped `CONFIG_TEMPLATE` points at Microsoft's own public client ID for
the **Microsoft Graph PowerShell** module:

```toml
[auth]
client_id = "14d82eec-204b-4c2f-b7e8-296a70dab67e"
tenant_id = "pitt.edu"
```

This app is:

- **First-party** (owned and published by Microsoft).
- **Multi-tenant**, so it works against any AAD tenant including Pitt.
- **Pre-configured** for the OAuth2 device code flow.
- Already consented to every Graph scope Prospectus Graphicus requests
  (`offline_access`, `User.Read`, `Mail.Read`, `Mail.ReadWrite`, `Mail.Send`).

Sign in:

```sh
prospectus auth login
```

You'll see Microsoft's sign-in page branded as **"Microsoft Graph PowerShell"**,
not Prospectus Graphicus — that's expected. If it succeeds, your refresh
token goes into the OS keyring and you can use the other commands normally.

### When Path 1 fails

If you see `AADSTS50105` ("administrator has configured the application ... to
block users unless they are specifically granted access"), Pitt has enabled
**assignment required** for the Microsoft Graph command-line first-party app.
Your account must be assigned to that enterprise application by an admin, or
you must use a different client ID/app registration. Move on to Path 2 or Path
3.

If you see `AADSTS65001` ("consent required" / "admin approval required") or
`AADSTS90094`, your tenant's admin has disabled user consent for this scope
set. Move on to Path 2.

Other alternatives at this tier, if Graph PowerShell is specifically
blocked in Pitt's tenant:

| Tool | Client ID |
|---|---|
| Azure CLI (`az`) | `04b07795-8ddb-461a-bbee-02f9e1bf7b46` |
| Azure PowerShell (`Az`) | `1950a258-227b-4e31-a9cf-717495945fc2` |

These are all Microsoft-published public clients. Swap the `client_id` and
re-run `prospectus auth login`.

### Tradeoffs of Path 1

- The consent screen isn't branded for Prospectus Graphicus — it says
  "Microsoft Graph PowerShell." Cosmetic but worth knowing.
- You're depending on Microsoft not changing the app's config. Stable in
  practice; no guarantee.
- Audit logs in Pitt's tenant will show this access as coming from the
  shared PowerShell app ID, not a Pitt-specific one. That's a compliance
  consideration if you ever share the tool.
- The Graph PowerShell scope surface is broad; consent grants more than
  Prospectus Graphicus strictly uses today. If that matters for your
  posture, go to Path 3.

## Path 2: Ask Pitt IT to consent or assign access

If Path 1 fails with a consent or assignment error, contact CSSD. A short,
specific request gets a much faster yes than "please register an app for me":

> Hi — I'd like to sign in to Microsoft Graph from a personal CLI using my
> Pitt account via the OAuth2 device code flow. I'm using the **Microsoft
> Graph PowerShell / Microsoft Graph Command Line Tools** client ID
> (`14d82eec-204b-4c2f-b7e8-296a70dab67e`) with delegated scopes
> `offline_access`, `User.Read`, `Mail.Read`, `Mail.ReadWrite`, `Mail.Send`.
> I received `AADSTS50105`, which says users are blocked unless assigned
> access. Could you either
>
> 1. assign my account to that enterprise application,
> 2. grant/admin-consent those delegated scopes for my account, or
> 3. enable user consent for apps from verified publishers?
>
> No new app registration is needed for options 1–3; I'm only using
> Microsoft's own first-party app.

If they can enable **(2)**, you unlock a whole class of tools with one
policy change.

## Path 3: Register your own Azure app

This is the officially correct path. Use it when:

- You need audit logs to attribute access to "Prospectus Graphicus"
  specifically.
- You want scopes narrower than Graph PowerShell's broad default.
- You're shipping this to other users in a regulated context.

Steps:

1. Sign in to <https://portal.azure.com> with your Pitt account.
2. Go to **Microsoft Entra ID → App registrations → New registration**.
3. Fill in:
   - **Name**: `Prospectus Graphicus`
   - **Supported account types**: *Accounts in this organizational directory
     only* (single tenant).
   - **Redirect URI**: leave blank.
4. **Register**. On the Overview page, copy **Application (client) ID** into
   `client_id`.
5. Under **Authentication → Advanced settings**, set **Allow public client
   flows** to **Yes**. Save.
6. Under **API permissions → Add a permission → Microsoft Graph → Delegated
   permissions**, add:
   - `offline_access`
   - `User.Read`
   - `Mail.Read`
   - `Mail.ReadWrite`
   - `Mail.Send`
7. If your tenant requires it, click **Grant admin consent for <tenant>**
   (needs admin role) or ask IT.

If the **New registration** button is greyed out or the save fails, Pitt has
disabled user self-service app registrations and you're stuck with Paths 1
or 2.

## Troubleshooting

- **`AADSTS50105` / "users are blocked unless assigned access"** — your
  tenant requires explicit assignment to the client application. Ask IT to
  assign your account to the Microsoft Graph command-line enterprise app, try
  another Microsoft first-party client ID, or register/use your own app.
- **`AADSTS7000218` / "client_assertion or client_secret required"** —
  *Allow public client flows* isn't enabled on your Path 3 app registration.
- **`AADSTS65001` / "consent required"** — the tenant requires admin consent
  for the requested scopes. Try Path 2.
- **`AADSTS50020` / "user does not exist in tenant"** — your `tenant_id`
  points at a tenant you're not a member of. Double-check, or use `common`.
- **`AADSTS700016` / "application not found in the directory"** — the
  `client_id` is wrong, or the app isn't multi-tenant and you're signing in
  from a different tenant.
