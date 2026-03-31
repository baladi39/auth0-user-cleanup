# CLI Usage Reference

## Synopsis

```
auth0-user-cleanup [OPTIONS]
```

Run with `cargo run -- [OPTIONS]` during development, or `./target/release/auth0-user-cleanup [OPTIONS]` after building.

## Options

| Flag | Value | Default | Description |
|------|-------|---------|-------------|
| `--dry-run` | *(none)* | off | Preview which resources would be deleted without making any changes. |
| `--domain <DOMAIN>` | Email domain (e.g. `example.com`) | *(all users)* | Only target users whose email ends with `@<DOMAIN>`. Can be specified multiple times. |
| `--resource <TYPE>` | `users` or `orgs` | `users` | Choose which Auth0 resource type to operate on. |
| `--name-pattern <PATTERN>` | String pattern | *(none)* | Filter organizations by name substring match (only applies when `--resource orgs`). |

## Examples

### Preview users that would be deleted

```bash
auth0-user-cleanup --dry-run
```

### Delete all users

```bash
auth0-user-cleanup
```

The tool will list all users and prompt for confirmation before deleting.

### Delete users filtered by email domain

```bash
# Single domain
auth0-user-cleanup --domain example.com

# Multiple domains
auth0-user-cleanup --domain example.com --domain test.com
```

### Combine dry run with domain filter

```bash
auth0-user-cleanup --dry-run --domain example.com
```

### Delete organizations

```bash
auth0-user-cleanup --resource orgs
```

### Delete organizations matching a name pattern

```bash
auth0-user-cleanup --resource orgs --name-pattern "test-"
```

### Dry run for organizations

```bash
auth0-user-cleanup --dry-run --resource orgs
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `AUTH0_DOMAIN` | Yes | Auth0 tenant domain (e.g. `your-tenant.us.auth0.com`) |
| `AUTH0_CLIENT_ID` | Yes | Client ID of the M2M application |
| `AUTH0_CLIENT_SECRET` | Yes | Client secret of the M2M application |
| `AUTH0_API_AUDIENCE` | No | Management API audience. Defaults to `https://<AUTH0_DOMAIN>/api/v2/` |

Configure these in a `.env` file at the project root.

## Behavior

- Users are fetched in pages of 100.
- Deletions are rate-limited (~2 req/sec, 500 ms delay) to stay within Auth0 free-tier limits.
- Users without an email are identified by `user_id` in output.
- Failed deletions are reported but do not stop the run; a summary is printed at the end.
- All destructive operations require interactive confirmation (`y/n` prompt) unless `--dry-run` is set.
