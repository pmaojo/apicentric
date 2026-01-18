use apicentric::ApicentricResult;
use colored::*;
use std::process::Command;

pub async fn doctor_command() -> ApicentricResult<()> {
    println!("{}", "🏥 Apicentric Doctor".bold().green());
    println!("Checking your system for potential issues...\n");

    let mut checks_passed = 0;
    let mut checks_failed = 0;

    // Check 1: apicentric version
    print!("Checking apicentric version... ");
    let version = env!("CARGO_PKG_VERSION");
    println!("{} v{}", "✅".green(), version);
    checks_passed += 1;

    // Check 2: rust/cargo installation
    print!("Checking Rust installation... ");
    match Command::new("cargo").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let s = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✅".green(), s.trim());
            checks_passed += 1;
        }
        _ => {
            println!("{} Cargo not found", "❌".red());
            checks_failed += 1;
        }
    }

    // Check 3: docker installation
    print!("Checking Docker installation... ");
    match Command::new("docker").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let s = String::from_utf8_lossy(&output.stdout);
            println!("{} {}", "✅".green(), s.trim());
            checks_passed += 1;
        }
        _ => {
            println!(
                "{} Docker not found (required for 'dockerize' command)",
                "⚠️".yellow()
            );
            // Warning implies mostly okay
            checks_passed += 1;
        }
    }

    // Check 4: Services directory
    print!("Checking services directory... ");
    if std::path::Path::new("services").exists() {
        println!("{} Found ./services", "✅".green());
        checks_passed += 1;
    } else {
        println!(
            "{} ./services not found (will be created on first use)",
            "ℹ️".blue()
        );
        checks_passed += 1;
    }

    // Check 5: Config file
    print!("Checking configuration... ");
    if std::path::Path::new("apicentric.json").exists() {
        println!("{} Found apicentric.json", "✅".green());
        checks_passed += 1;
    } else {
        println!("{} apicentric.json not found (using defaults)", "ℹ️".blue());
        checks_passed += 1;
    }

    println!();
    if checks_failed == 0 {
        println!(
            "✨ All systems operational! ({} checks passed) You are ready to build.",
            checks_passed
        );
    } else {
        println!(
            "⚠️  Some issues were found ({} checks passed, {} failed). Please review the errors above.",
            checks_passed, checks_failed
        );
    }

    Ok(())
}
