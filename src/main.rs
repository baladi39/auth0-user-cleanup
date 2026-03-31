use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct User {
    user_id: String,
    email: Option<String>,
}

#[derive(Deserialize)]
struct UsersPage {
    users: Vec<User>,
    total: u64,
}

#[derive(Deserialize)]
struct Organization {
    id: String,
    name: String,
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct OrganizationsPage {
    organizations: Vec<Organization>,
    total: u64,
}

async fn get_management_token(client: &Client, domain: &str, client_id: &str, client_secret: &str, audience: &str) -> anyhow::Result<String> {
    let body = serde_json::json!({
        "grant_type": "client_credentials",
        "client_id": client_id,
        "client_secret": client_secret,
        "audience": audience,
    });

    let response = client
        .post(format!("https://{}/oauth/token", domain))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Token request failed ({status}): {body}");
    }

    let res: TokenResponse = response.json().await?;

    Ok(res.access_token)
}

async fn fetch_all_users(client: &Client, domain: &str, token: &str) -> anyhow::Result<Vec<User>> {
    let mut users = Vec::new();
    let mut page = 0u32;
    let per_page = 100u32;

    loop {
        let res: UsersPage = client
            .get(format!("https://{}/api/v2/users", domain))
            .bearer_auth(token)
            .query(&[
                ("per_page", per_page.to_string()),
                ("page", page.to_string()),
                ("include_totals", "true".to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let fetched = res.users.len();
        println!("Page {page}: fetched {fetched} users (total so far: {})", users.len() + fetched);
        users.extend(res.users);

        if users.len() as u64 >= res.total {
            break;
        }
        page += 1;
    }

    Ok(users)
}

async fn delete_user(client: &Client, domain: &str, token: &str, user_id: &str) -> anyhow::Result<()> {
    client
        .delete(format!(
            "https://{}/api/v2/users/{}",
            domain,
            urlencoding::encode(user_id)
        ))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn fetch_all_organizations(client: &Client, domain: &str, token: &str) -> anyhow::Result<Vec<Organization>> {
    let mut orgs = Vec::new();
    let mut page = 0u32;
    let per_page = 100u32;

    loop {
        let res: OrganizationsPage = client
            .get(format!("https://{}/api/v2/organizations", domain))
            .bearer_auth(token)
            .query(&[
                ("per_page", per_page.to_string()),
                ("page", page.to_string()),
                ("include_totals", "true".to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let fetched = res.organizations.len();
        println!("Page {page}: fetched {fetched} organizations (total so far: {})", orgs.len() + fetched);
        orgs.extend(res.organizations);

        if orgs.len() as u64 >= res.total {
            break;
        }
        page += 1;
    }

    Ok(orgs)
}

async fn delete_organization(client: &Client, domain: &str, token: &str, org_id: &str) -> anyhow::Result<()> {
    client
        .delete(format!(
            "https://{}/api/v2/organizations/{}",
            domain,
            urlencoding::encode(org_id)
        ))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let mut domains: Vec<String> = Vec::new();
    let mut resource = String::from("users");
    let mut name_pattern: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--domain" {
            if let Some(d) = args.get(i + 1) {
                domains.push(d.clone());
                i += 2;
                continue;
            }
        } else if args[i] == "--resource" {
            if let Some(r) = args.get(i + 1) {
                resource = r.clone();
                i += 2;
                continue;
            }
        } else if args[i] == "--name-pattern" {
            if let Some(p) = args.get(i + 1) {
                name_pattern = Some(p.clone());
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    if resource != "users" && resource != "orgs" {
        anyhow::bail!("Invalid --resource value: '{resource}'. Must be 'users' or 'orgs'.");
    }

    let domain = env::var("AUTH0_DOMAIN").expect("AUTH0_DOMAIN must be set");
    let client_id = env::var("AUTH0_CLIENT_ID").expect("AUTH0_CLIENT_ID must be set");
    let client_secret = env::var("AUTH0_CLIENT_SECRET").expect("AUTH0_CLIENT_SECRET must be set");
    let audience = env::var("AUTH0_API_AUDIENCE")
        .unwrap_or_else(|_| format!("https://{}/api/v2/", domain));

    let client = Client::new();

    if dry_run {
        println!("[DRY RUN] No changes will be made.\n");
    }

    println!("Connecting to tenant: {domain}");
    let token = get_management_token(&client, &domain, &client_id, &client_secret, &audience).await?;

    if resource == "orgs" {
        // ── Organization deletion flow ──
        if let Some(ref pat) = name_pattern {
            println!("Filtering organizations by name pattern: {pat}");
        }

        println!("Fetching all organizations...");
        let all_orgs = fetch_all_organizations(&client, &domain, &token).await?;
        let fetched_total = all_orgs.len();

        let orgs: Vec<Organization> = if let Some(ref pat) = name_pattern {
            all_orgs.into_iter().filter(|o| o.name.contains(pat.as_str())).collect()
        } else {
            all_orgs
        };

        let total = orgs.len();
        if name_pattern.is_some() {
            println!("Found {fetched_total} total organizations, {total} match the name filter.");
        }

        println!("\nFound {total} organizations. The following will be deleted:\n");
        for org in &orgs {
            let display = org.display_name.as_deref().unwrap_or(&org.name);
            println!("  - {display} ({}) [{}]", org.name, org.id);
        }

        if dry_run {
            println!("\n[DRY RUN] {total} organizations would be deleted. No changes were made.");
            return Ok(());
        }

        println!("\nAre you sure you want to delete {total} organizations? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted. No organizations were deleted.");
            return Ok(());
        }

        println!("\nStarting deletion...\n");

        let mut deleted = 0usize;
        let mut failed = 0usize;

        for org in &orgs {
            let label = org.display_name.as_deref().unwrap_or(&org.name);
            match delete_organization(&client, &domain, &token, &org.id).await {
                Ok(()) => {
                    deleted += 1;
                    println!("[{deleted}/{total}] Deleted: {label} ({})", org.id);
                }
                Err(e) => {
                    eprintln!("  ERROR deleting {label} ({}): {e}", org.id);
                    failed += 1;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        println!("\nDone. Deleted: {deleted}, Failed: {failed}");
    } else {
        // ── User deletion flow ──
        if !domains.is_empty() {
            println!("Filtering by domain(s): {}", domains.join(", "));
        }

        println!("Fetching all users...");
        let all_users = fetch_all_users(&client, &domain, &token).await?;
        let fetched_total = all_users.len();

        let users: Vec<User> = if domains.is_empty() {
            all_users
        } else {
            all_users.into_iter().filter(|u| {
                u.email.as_deref()
                    .map(|e| domains.iter().any(|d| e.ends_with(&format!("@{d}"))))
                    .unwrap_or(false)
            }).collect()
        };

        let total = users.len();
        if !domains.is_empty() {
            println!("Found {fetched_total} total users, {total} match the domain filter.");
        }

        println!("\nFound {total} users. The following will be deleted:\n");
        for user in &users {
            let label = user.email.as_deref().unwrap_or(&user.user_id);
            println!("  - {label} ({})", user.user_id);
        }

        if dry_run {
            println!("\n[DRY RUN] {total} users would be deleted. No changes were made.");
            return Ok(());
        }

        println!("\nAre you sure you want to delete {total} users? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Aborted. No users were deleted.");
            return Ok(());
        }

        println!("\nStarting deletion...\n");

        let mut deleted = 0usize;
        let mut failed = 0usize;

        for user in users {
            let label = user.email.as_deref().unwrap_or(&user.user_id).to_string();
            match delete_user(&client, &domain, &token, &user.user_id).await {
                Ok(()) => {
                    deleted += 1;
                    println!("[{deleted}/{total}] Deleted: {label}");
                }
                Err(e) => {
                    eprintln!("  ERROR deleting {label}: {e}");
                    failed += 1;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        println!("\nDone. Deleted: {deleted}, Failed: {failed}");
    }

    Ok(())
}
