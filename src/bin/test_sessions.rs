use backplane_tui::claude::{find_claude_data_path, load_usage_entries, analyze_sessions};

fn main() {
    println!("Testing Claude session loading...\n");

    match find_claude_data_path() {
        Some(path) => {
            println!("Data path: {:?}", path);

            // Show details for 30-day window
            if let Ok(entries) = load_usage_entries(&path, 720) {
                let sessions = analyze_sessions(entries);
                println!("Found {} sessions\n", sessions.len());

                for (i, s) in sessions.iter().enumerate() {
                    println!("[{}] {} - {}", i, &s.session_id[..s.session_id.len().min(12)], s.short_name(30));
                    println!("    cwd: {}", s.cwd);
                    println!("    exists: {}", std::path::Path::new(&s.cwd).is_dir());
                    println!();
                }
            }
        }
        None => {
            println!("No Claude data path found");
        }
    }
}
