//! Simple Navidrome Connection Test
//!
//! A standalone binary to test Navidrome connectivity and authentication.
//! Run with: cargo run --bin navidrome_test

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 StepheyBot Music - Navidrome Connection Test");
    println!("==============================================");
    println!();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get Navidrome configuration
    let navidrome_url = env::var("NAVIDROME_URL").unwrap_or_default();
    let username = env::var("NAVIDROME_USERNAME").unwrap_or_default();
    let password = env::var("NAVIDROME_PASSWORD").unwrap_or_default();

    // Check if configuration is present
    if navidrome_url.is_empty() || username.is_empty() || password.is_empty() {
        println!("❌ Navidrome not configured");
        println!("   Please set these environment variables:");
        println!("   NAVIDROME_URL=http://localhost:4533");
        println!("   NAVIDROME_USERNAME=your_username");
        println!("   NAVIDROME_PASSWORD=your_password");
        println!();
        println!("   You can create a .env file with these variables");
        return Ok(());
    }

    println!("📋 Configuration:");
    println!("   URL: {}", navidrome_url);
    println!("   Username: {}", username);
    println!("   Password: [HIDDEN]");
    println!();

    // Test 1: Basic connectivity
    println!("🌐 Test 1: Basic connectivity...");
    let client = reqwest::Client::new();

    match client.get(&navidrome_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("   ✅ Server is reachable (HTTP {})", response.status());
            } else {
                println!("   ⚠️  Server responded with HTTP {}", response.status());
            }
        }
        Err(e) => {
            println!("   ❌ Connection failed: {}", e);
            return Ok(());
        }
    }

    // Test 2: Authentication
    println!("🔐 Test 2: Testing authentication...");

    // Generate salt and token for Subsonic API
    let salt = "randomsalt";
    let token = format!("{:x}", md5::compute(format!("{}{}", password, salt)));

    let auth_params = format!(
        "u={}&t={}&s={}&v=1.16.1&c=StepheyBot-Music",
        username, token, salt
    );

    let ping_url = format!("{}/rest/ping?{}", navidrome_url, auth_params);

    match client.get(&ping_url).send().await {
        Ok(response) => {
            let text = response.text().await?;
            if text.contains("status=\"ok\"") {
                println!("   ✅ Authentication successful");
            } else if text.contains("status=\"failed\"") {
                println!("   ❌ Authentication failed");
                if text.contains("code=\"40\"") {
                    println!("       Wrong username or password");
                } else {
                    println!("       Server response: {}", text);
                }
                return Ok(());
            } else {
                println!("   ⚠️  Unexpected response: {}", text);
            }
        }
        Err(e) => {
            println!("   ❌ Authentication test failed: {}", e);
            return Ok(());
        }
    }

    // Test 3: Get library info
    println!("📊 Test 3: Getting library information...");

    let artists_url = format!("{}/rest/getArtists?{}", navidrome_url, auth_params);

    match client.get(&artists_url).send().await {
        Ok(response) => {
            let text = response.text().await?;
            if text.contains("status=\"ok\"") {
                // Simple count of artists
                let artist_count = text.matches("<artist").count();
                println!("   ✅ Found {} artist(s) in library", artist_count);

                if artist_count > 0 {
                    println!("   🎵 Library is populated and accessible");
                } else {
                    println!("   ⚠️  Library appears to be empty or scanning");
                }
            } else {
                println!("   ❌ Failed to get library info");
            }
        }
        Err(e) => {
            println!("   ❌ Library test failed: {}", e);
        }
    }

    // Test 4: Get random songs
    println!("🎲 Test 4: Testing random songs endpoint...");

    let random_url = format!(
        "{}/rest/getRandomSongs?size=3&{}",
        navidrome_url, auth_params
    );

    match client.get(&random_url).send().await {
        Ok(response) => {
            let text = response.text().await?;
            if text.contains("status=\"ok\"") {
                let song_count = text.matches("<song").count();
                println!("   ✅ Retrieved {} random song(s)", song_count);
            } else {
                println!("   ❌ Failed to get random songs");
            }
        }
        Err(e) => {
            println!("   ❌ Random songs test failed: {}", e);
        }
    }

    println!();
    println!("🎉 Navidrome connection test completed!");
    println!();
    println!("📋 Next steps:");
    println!("   1. Run: ./enable-navidrome.sh");
    println!("   2. Start StepheyBot Music: ./start-dev.sh");
    println!("   3. Test integration: curl http://localhost:8083/api/v1/navidrome/status");
    println!();

    Ok(())
}
