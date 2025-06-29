use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn setup_ssh_auth() {
    println!("🔐 SSH Authentication Setup");
    println!("Setting up SSH authentication for Git operations...\n");

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let ssh_dir = Path::new(&home_dir).join(".ssh");

    // Create .ssh directory if it doesn't exist
    if !ssh_dir.exists() {
        println!("📁 Creating .ssh directory...");
        if let Err(e) = fs::create_dir_all(&ssh_dir) {
            eprintln!("❌ Failed to create .ssh directory: {}", e);
            return;
        }
        println!("✅ .ssh directory created successfully");
    }

    // Check for existing SSH keys
    let key_types = [
        ("id_ed25519", "Ed25519 (recommended)"),
        ("id_rsa", "RSA"),
        ("id_ecdsa", "ECDSA"),
    ];

    let mut existing_keys = Vec::new();
    for (key_name, key_type) in &key_types {
        let private_key = ssh_dir.join(key_name);
        let public_key = ssh_dir.join(format!("{}.pub", key_name));

        if private_key.exists() && public_key.exists() {
            // Read the public key to extract identity
            let identity = match fs::read_to_string(&public_key) {
                Ok(content) => {
                    // Extract the comment/email part (last part after the key data)
                    let parts: Vec<&str> = content.split_whitespace().collect();
                    if parts.len() >= 3 {
                        parts[2..].join(" ")
                    } else {
                        "No identity found".to_string()
                    }
                }
                Err(_) => "Could not read key".to_string(),
            };

            existing_keys.push((*key_name, *key_type, public_key, identity));
        }
    }

    if !existing_keys.is_empty() {
        println!("🔍 Found existing SSH keys:");
        for (i, (key_name, key_type, _, identity)) in existing_keys.iter().enumerate() {
            println!(" {}. {} ({}) - {}", i + 1, key_name, key_type, identity);
        }

        let options = vec!["Use existing key", "Generate new key", "Exit"];
        match Select::new()
            .with_prompt("Choose an option")
            .default(0)
            .items(&options)
            .interact()
        {
            Ok(0) => {
                if existing_keys.len() == 1 {
                    display_public_key_and_guide(&existing_keys[0].2);
                } else {
                    select_existing_key(&existing_keys);
                }
                return;
            }
            Ok(1) => {
                // Continue to generate new key
            }
            Ok(2) | Err(_) => {
                println!("Setup cancelled.");
                return;
            }
            _ => unreachable!(),
        }
    }

    // Generate new SSH key
    generate_new_ssh_key(&ssh_dir);
}

fn select_existing_key(existing_keys: &[(&str, &str, std::path::PathBuf, String)]) {
    let key_options: Vec<String> = existing_keys
        .iter()
        .map(|(key_name, key_type, _, identity)| {
            format!("{} ({}) - {}", key_name, key_type, identity)
        })
        .collect();

    match Select::new()
        .with_prompt("Select which key to use")
        .default(0)
        .items(&key_options)
        .interact()
    {
        Ok(choice) => {
            display_public_key_and_guide(&existing_keys[choice].2);
        }
        Err(_) => {
            println!("Selection cancelled.");
        }
    }
}

fn generate_new_ssh_key(ssh_dir: &Path) {
    println!("\n🔑 Generating new SSH key...");

    // Get user email
    let email = match Input::<String>::new()
        .with_prompt("Enter your email address")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Email cannot be empty")
            } else if !input.contains('@') {
                Err("Please enter a valid email address")
            } else {
                Ok(())
            }
        })
        .interact()
    {
        Ok(email) => email.trim().to_string(),
        Err(_) => {
            println!("Email input cancelled. Exiting.");
            return;
        }
    };

    // Choose key type
    let key_types = vec![
        "Ed25519 (recommended, modern and secure)",
        "RSA 4096 (widely compatible)",
    ];

    let key_choice = match Select::new()
        .with_prompt("Choose SSH key type")
        .default(0)
        .items(&key_types)
        .interact()
    {
        Ok(choice) => choice,
        Err(_) => {
            println!("Key type selection cancelled. Exiting.");
            return;
        }
    };

    let (key_type, key_name, ssh_keygen_args) = match key_choice {
        1 => ("RSA", "id_rsa", vec!["-t", "rsa", "-b", "4096"]),
        _ => ("Ed25519", "id_ed25519", vec!["-t", "ed25519"]),
    };

    println!("\n🔧 Generating {} key...", key_type);
    let key_path = ssh_dir.join(key_name);

    // Build ssh-keygen command
    let mut cmd = Command::new("ssh-keygen");
    cmd.args(&ssh_keygen_args)
        .arg("-C")
        .arg(&email)
        .arg("-f")
        .arg(&key_path)
        .arg("-N")
        .arg(""); // Empty passphrase for simplicity

    match cmd.status() {
        Ok(status) if status.success() => {
            println!("✅ SSH key generated successfully!");
            // Add to ssh-agent
            add_key_to_agent(&key_path);
            // Display public key and guide
            let public_key_path = ssh_dir.join(format!("{}.pub", key_name));
            display_public_key_and_guide(&public_key_path);
        }
        Ok(status) => {
            eprintln!("❌ ssh-keygen failed with status: {}", status);
        }
        Err(e) => {
            eprintln!("❌ Failed to run ssh-keygen: {}", e);
            eprintln!("Make sure OpenSSH is installed on your system.");
        }
    }
}

fn add_key_to_agent(key_path: &Path) {
    println!("\n🔧 Adding key to ssh-agent...");

    // Check if ssh-agent is running
    if std::env::var("SSH_AUTH_SOCK").is_err() {
        println!("SSH agent is not running. Starting ssh-agent...");
        // Try to start ssh-agent
        match Command::new("ssh-agent").arg("-s").output() {
            Ok(output) if output.status.success() => {
                let agent_output = String::from_utf8_lossy(&output.stdout);
                println!("SSH agent started. You may need to run the following commands:");
                println!("{}", agent_output.trim());
            }
            _ => {
                println!("⚠️ Could not start ssh-agent automatically.");
                println!("You may need to start it manually with: eval $(ssh-agent -s)");
            }
        }
    }

    // Add key to agent
    match Command::new("ssh-add").arg(key_path).status() {
        Ok(status) if status.success() => {
            println!("✅ Key added to ssh-agent successfully!");
        }
        Ok(_) => {
            println!("⚠️ Failed to add key to ssh-agent");
            println!(
                "💡 You can add it manually with: ssh-add {}",
                key_path.display()
            );
        }
        Err(_) => {
            println!("⚠️ ssh-add not available");
            println!(
                "💡 You can add it manually later with: ssh-add {}",
                key_path.display()
            );
        }
    }
}

fn display_public_key_and_guide(public_key_path: &Path) {
    println!("\n📋 Your SSH Public Key:");
    println!("{}", "─".repeat(60));

    match fs::read_to_string(public_key_path) {
        Ok(public_key) => {
            println!("{}", public_key.trim());
            println!("{}", "─".repeat(60));

            println!("\n🚀 Next Steps:");
            println!("\n1. Copy the public key above (it's already selected for you)");
            println!("\n2. Add it to your GitHub account:");
            println!(" • Go to: https://github.com/settings/ssh/new");
            println!(" • Or navigate to: GitHub → Settings → SSH and GPG keys → New SSH key");
            println!("\n3. Fill in the form:");
            println!(" • Title: Give it a descriptive name (e.g., 'My Laptop - bgit')");
            println!(" • Key type: Authentication Key");
            println!(" • Key: Paste the public key from above");
            println!("\n4. Click 'Add SSH key' and enter your GitHub password if prompted");
            println!("\n5. Test your connection:");
            println!(" ssh -T git@github.com");
            println!(
                "\n🎉 You're all set! Your bgit tool can now authenticate with GitHub using SSH."
            );

            // Offer to open GitHub in browser
            if Confirm::new()
                .with_prompt("Would you like to open GitHub SSH settings in your default browser?")
                .default(false)
                .interact()
                .unwrap_or(false)
            {
                open_github_ssh_settings();
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to read public key file: {}", e);
        }
    }
}

fn open_github_ssh_settings() {
    let url = "https://github.com/settings/ssh/new";

    #[cfg(target_os = "windows")]
    let cmd = Command::new("cmd").args(["/c", "start", url]).status();

    #[cfg(target_os = "macos")]
    let cmd = Command::new("open").arg(url).status();

    #[cfg(target_os = "linux")]
    let cmd = Command::new("xdg-open").arg(url).status();

    match cmd {
        Ok(status) if status.success() => {
            println!("🌐 Opening GitHub SSH settings in your browser...");
        }
        _ => {
            println!("⚠️ Could not open browser automatically.");
            println!("🔗 Please visit: {}", url);
        }
    }
}
