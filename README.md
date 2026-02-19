# auth0-user-cleanup

A Rust CLI tool to bulk-delete users from an Auth0 tenant via the Management API. Supports dry-run mode and optional filtering by email domain.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024, Rust 1.85+)
- An Auth0 Machine-to-Machine application with the **Auth0 Management API** authorized and the `delete:users` / `read:users` scopes granted

## Setup

### 1. Clone the repository

```bash
git clone <repo-url>
cd auth0-user-cleanup
```

### 2. Create the `.env` file

Copy the example below and fill in your Auth0 credentials:

```bash
cp .env.example .env   # or create .env manually
```

`.env` contents:

```env
AUTH0_DOMAIN=your-tenant.region.auth0.com
AUTH0_CLIENT_ID=your_client_id
AUTH0_CLIENT_SECRET=your_client_secret

# Optional — defaults to https://<AUTH0_DOMAIN>/api/v2/
AUTH0_API_AUDIENCE=https://your-tenant.region.auth0.com/api/v2/
```

### 3. Build the project

```bash
cargo build --release
```

The compiled binary will be at `./target/release/auth0-user-cleanup`.

## Usage

### Dry run (preview only — no deletions)

Always run this first to see which users would be affected.

```bash
cargo run -- --dry-run
```

Or with the release binary:

```bash
./target/release/auth0-user-cleanup --dry-run
```

### Delete all users

```bash
cargo run
```

```bash
./target/release/auth0-user-cleanup
```

### Filter by email domain

Only delete users whose email matches the specified domain(s). Pass `--domain` once per domain.

```bash
# Single domain
cargo run -- --domain example.com

# Multiple domains
cargo run -- --domain example.com --domain test.com
```

### Combine filters

```bash
# Dry run scoped to a domain
cargo run -- --dry-run --domain example.com
```

## Environment Variables

| Variable             | Required | Description                                                                 |
|----------------------|----------|-----------------------------------------------------------------------------|
| `AUTH0_DOMAIN`       | Yes      | Your Auth0 tenant domain, e.g. `your-tenant.us.auth0.com`                  |
| `AUTH0_CLIENT_ID`    | Yes      | Client ID of your M2M application                                           |
| `AUTH0_CLIENT_SECRET`| Yes      | Client secret of your M2M application                                       |
| `AUTH0_API_AUDIENCE` | No       | Audience for the Management API token. Defaults to `https://<AUTH0_DOMAIN>/api/v2/` |

## Notes

- The tool fetches users in pages of 100. Progress is printed per page.
- Deletions are rate-limited to ~2 requests/second (500 ms delay between calls) to stay within Auth0 free-tier limits.
- Users without an email address are identified by their `user_id` in output.
- Failed deletions are reported but do not stop the run; a summary is printed at the end.
