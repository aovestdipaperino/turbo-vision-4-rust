// (C) 2025 - Enzo Lombardi
// Test program to debug file filtering

use std::fs;
use std::path::PathBuf;

fn matches_wildcard(wildcard: &str, name: &str) -> bool {
    if wildcard == "*" || wildcard.is_empty() {
        return true;
    }

    // Simple wildcard matching (*.ext)
    if let Some(ext) = wildcard.strip_prefix("*.") {
        name.ends_with(&format!(".{}", ext))
    } else {
        name.contains(wildcard)
    }
}

fn main() {
    let current_path = PathBuf::from(".");
    let wildcard = "*.rs";

    println!("Reading directory: {}", current_path.display());
    println!("Wildcard: {}", wildcard);
    println!();

    if let Ok(entries) = fs::read_dir(&current_path) {
        let mut dirs = Vec::new();
        let mut regular_files = Vec::new();

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();

                if metadata.is_dir() {
                    dirs.push(format!("[{}]", name));
                    println!("DIR: {}", name);
                } else {
                    let matches = matches_wildcard(wildcard, &name);
                    println!("FILE: {} - matches: {}", name, matches);
                    if matches {
                        regular_files.push(name);
                    }
                }
            }
        }

        println!("\n=== Final lists ===");
        println!("Directories: {:?}", dirs);
        println!("Files: {:?}", regular_files);

        // Sort and combine
        dirs.sort();
        regular_files.sort();
        let mut all_files = dirs;
        all_files.extend(regular_files);

        println!("\n=== Combined list ===");
        for file in &all_files {
            println!("  {}", file);
        }
    }
}
