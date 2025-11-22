//! Authentication and RBAC command handlers

use anyhow::{Context, Result};
use bazbom::cli::{AuthTokenCmd, AuthUserCmd};
use std::path::PathBuf;

/// Handle auth init
pub fn handle_auth_init() -> Result<()> {
    println!("Initializing BazBOM authentication system...\n");

    let auth_dir = get_auth_dir()?;
    std::fs::create_dir_all(&auth_dir)?;

    // Create initial config
    let config = serde_json::json!({
        "version": "1.0",
        "created": chrono::Utc::now().to_rfc3339(),
        "jwt_secret": generate_secret(),
        "token_lifetime_hours": 24,
        "api_key_lifetime_days": 365,
    });

    let config_path = auth_dir.join("config.json");
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

    // Create empty users file
    let users_path = auth_dir.join("users.json");
    std::fs::write(&users_path, "[]")?;

    // Create empty tokens file
    let tokens_path = auth_dir.join("tokens.json");
    std::fs::write(&tokens_path, "[]")?;

    // Create audit log
    let audit_path = auth_dir.join("audit.json");
    std::fs::write(&audit_path, "[]")?;

    println!("[+] Authentication system initialized!");
    println!("    Config: {}", config_path.display());
    println!("    Users:  {}", users_path.display());
    println!("    Tokens: {}", tokens_path.display());
    println!("    Audit:  {}", audit_path.display());
    println!("\nNext steps:");
    println!("  1. Add users: bazbom auth user add admin@example.com --role admin");
    println!("  2. Create tokens: bazbom auth token create --name ci-token --scope read");

    Ok(())
}

/// Handle user management commands
pub fn handle_auth_user(cmd: AuthUserCmd) -> Result<()> {
    let auth_dir = get_auth_dir()?;
    let users_path = auth_dir.join("users.json");

    ensure_initialized(&auth_dir)?;

    match cmd {
        AuthUserCmd::Add { email, role } => {
            let mut users = load_users(&users_path)?;

            // Check if user exists
            if users
                .iter()
                .any(|u: &serde_json::Value| u["email"] == email)
            {
                anyhow::bail!("User already exists: {}", email);
            }

            // Add user
            users.push(serde_json::json!({
                "email": email,
                "role": role,
                "created": chrono::Utc::now().to_rfc3339(),
                "active": true,
            }));

            save_users(&users_path, &users)?;
            println!("[+] User added: {} (role: {})", email, role);
        }
        AuthUserCmd::List {} => {
            let users = load_users(&users_path)?;

            if users.is_empty() {
                println!("No users configured.");
                println!("Add a user with: bazbom auth user add <email> --role <role>");
                return Ok(());
            }

            println!("Users:\n");
            println!("{:<30} {:<15} {:<10}", "EMAIL", "ROLE", "STATUS");
            println!("{}", "-".repeat(55));

            for user in &users {
                let email = user["email"].as_str().unwrap_or("?");
                let role = user["role"].as_str().unwrap_or("?");
                let active = if user["active"].as_bool().unwrap_or(true) {
                    "active"
                } else {
                    "disabled"
                };
                println!("{:<30} {:<15} {:<10}", email, role, active);
            }
        }
        AuthUserCmd::Remove { email } => {
            let mut users = load_users(&users_path)?;
            let original_len = users.len();

            users.retain(|u: &serde_json::Value| u["email"] != email);

            if users.len() == original_len {
                anyhow::bail!("User not found: {}", email);
            }

            save_users(&users_path, &users)?;
            println!("[+] User removed: {}", email);
        }
        AuthUserCmd::SetRole { email, role } => {
            let mut users = load_users(&users_path)?;
            let mut found = false;

            for user in users.iter_mut() {
                if user["email"] == email {
                    user["role"] = serde_json::Value::String(role.clone());
                    found = true;
                    break;
                }
            }

            if !found {
                anyhow::bail!("User not found: {}", email);
            }

            save_users(&users_path, &users)?;
            println!("[+] Updated role for {}: {}", email, role);
        }
    }

    Ok(())
}

/// Handle token management commands
pub fn handle_auth_token(cmd: AuthTokenCmd) -> Result<()> {
    let auth_dir = get_auth_dir()?;
    let tokens_path = auth_dir.join("tokens.json");

    ensure_initialized(&auth_dir)?;

    match cmd {
        AuthTokenCmd::Create {
            name,
            scope,
            expires,
        } => {
            let mut tokens = load_tokens(&tokens_path)?;

            // Generate token
            let token_id = generate_token_id();
            let token_value = generate_token_value();

            let expiry = if expires > 0 {
                Some((chrono::Utc::now() + chrono::Duration::days(expires as i64)).to_rfc3339())
            } else {
                None
            };

            tokens.push(serde_json::json!({
                "id": token_id,
                "name": name,
                "scope": scope,
                "created": chrono::Utc::now().to_rfc3339(),
                "expires": expiry,
                "last_used": null,
            }));

            save_tokens(&tokens_path, &tokens)?;

            println!("[+] API token created!");
            println!("\n    Token ID: {}", token_id);
            println!("    Name:     {}", name);
            println!("    Scope:    {}", scope);
            if let Some(exp) = expiry {
                println!("    Expires:  {}", exp);
            } else {
                println!("    Expires:  never");
            }
            println!("\n    Token: {}", token_value);
            println!("\n    [!] Save this token now - it cannot be retrieved later!");
        }
        AuthTokenCmd::List {} => {
            let tokens = load_tokens(&tokens_path)?;

            if tokens.is_empty() {
                println!("No tokens configured.");
                println!(
                    "Create a token with: bazbom auth token create --name <name> --scope <scope>"
                );
                return Ok(());
            }

            println!("API Tokens:\n");
            println!(
                "{:<12} {:<20} {:<10} {:<20}",
                "ID", "NAME", "SCOPE", "EXPIRES"
            );
            println!("{}", "-".repeat(62));

            for token in &tokens {
                let id = token["id"].as_str().unwrap_or("?");
                let name = token["name"].as_str().unwrap_or("?");
                let scope = token["scope"].as_str().unwrap_or("?");
                let expires = token["expires"].as_str().unwrap_or("never");
                println!("{:<12} {:<20} {:<10} {:<20}", id, name, scope, expires);
            }
        }
        AuthTokenCmd::Revoke { token_id } => {
            let mut tokens = load_tokens(&tokens_path)?;
            let original_len = tokens.len();

            tokens.retain(|t: &serde_json::Value| t["id"] != token_id);

            if tokens.len() == original_len {
                anyhow::bail!("Token not found: {}", token_id);
            }

            save_tokens(&tokens_path, &tokens)?;
            println!("[+] Token revoked: {}", token_id);
        }
    }

    Ok(())
}

/// Handle audit log viewing
pub fn handle_auth_audit_log(limit: usize, event_type: Option<String>) -> Result<()> {
    let auth_dir = get_auth_dir()?;
    let audit_path = auth_dir.join("audit.json");

    ensure_initialized(&auth_dir)?;

    let content = std::fs::read_to_string(&audit_path).context("Failed to read audit log")?;
    let events: Vec<serde_json::Value> = serde_json::from_str(&content)?;

    let filtered: Vec<_> = events
        .iter()
        .filter(|e| {
            if let Some(ref filter) = event_type {
                e["type"].as_str().map(|t| t == filter).unwrap_or(false)
            } else {
                true
            }
        })
        .rev()
        .take(limit)
        .collect();

    if filtered.is_empty() {
        println!("No audit events found.");
        return Ok(());
    }

    println!("Audit Log (last {} events):\n", filtered.len());
    println!(
        "{:<20} {:<15} {:<15} {}",
        "TIMESTAMP", "TYPE", "USER", "DETAILS"
    );
    println!("{}", "-".repeat(70));

    for event in &filtered {
        let timestamp = event["timestamp"]
            .as_str()
            .unwrap_or("?")
            .chars()
            .take(19)
            .collect::<String>();
        let event_type = event["type"].as_str().unwrap_or("?");
        let user = event["user"].as_str().unwrap_or("system");
        let details = event["details"].as_str().unwrap_or("");

        println!(
            "{:<20} {:<15} {:<15} {}",
            timestamp, event_type, user, details
        );
    }

    Ok(())
}

// Helper functions

fn get_auth_dir() -> Result<PathBuf> {
    let dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".config"))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("bazbom")
        .join("auth");
    Ok(dir)
}

fn ensure_initialized(auth_dir: &PathBuf) -> Result<()> {
    if !auth_dir.exists() {
        anyhow::bail!("Authentication system not initialized. Run: bazbom auth init");
    }
    Ok(())
}

fn load_users(path: &PathBuf) -> Result<Vec<serde_json::Value>> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_users(path: &PathBuf, users: &Vec<serde_json::Value>) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(users)?)?;
    Ok(())
}

fn load_tokens(path: &PathBuf) -> Result<Vec<serde_json::Value>> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_tokens(path: &PathBuf, tokens: &Vec<serde_json::Value>) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(tokens)?)?;
    Ok(())
}

fn generate_secret() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("bzb_{:x}", nanos)
}

fn generate_token_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("tk_{:x}", millis % 0xFFFFFFFF)
}

fn generate_token_value() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("bzb_live_{:x}", nanos)
}
