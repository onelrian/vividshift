use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};

/// Format assignments into a readable Discord message
fn format_assignments_message(assignments: &HashMap<String, Vec<String>>) -> String {
    let mut message = String::from("**🔄 New Work Assignments Generated!**\n\n");

    // Sort areas alphabetically for consistency
    let mut areas: Vec<_> = assignments.keys().collect();
    areas.sort();

    for area in areas {
        if let Some(people) = assignments.get(area) {
            let people_list = people.join(", ");
            // Create a formatted line: **Area**: Person1, Person2
            message.push_str(&format!("**{}**: {}\n", area, people_list));
        }
    }

    message.push_str("\n*Please check the full schedule for details.*");
    message
}

/// Send notification to Discord webhook
pub fn send_notification(webhook_url: &str, assignments: &HashMap<String, Vec<String>>) -> Result<()> {
    info!("🚀 Attempting to send Discord notification...");
    
    let content = format_assignments_message(assignments);
    
    // Create the payload
    // Discord webhooks accept a JSON body with a "content" field
    let payload = json!({
        "content": content
    });

    let client = Client::new();
    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .context("Failed to send request to Discord webhook")?;

    if response.status().is_success() {
        info!("✅ Discord notification sent successfully.");
    } else {
        let status = response.status();
        let text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        error!("❌ Discord Webhook failed. Status: {}, Response: {}", status, text);
        anyhow::bail!("Discord API returned error: {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_format_assignments_message() {
        let mut assignments = HashMap::new();
        assignments.insert("Kitchen".to_string(), vec!["Alice".to_string(), "Bob".to_string()]);
        assignments.insert("Garden".to_string(), vec!["Charlie".to_string()]);

        let message = format_assignments_message(&assignments);
        
        println!("{}", message); // For visual debugging if needed

        assert!(message.contains("**🔄 New Work Assignments Generated!**"));
        assert!(message.contains("**Kitchen**: Alice, Bob"));
        assert!(message.contains("**Garden**: Charlie"));
    }
}
