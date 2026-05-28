use std::process::Command;

fn main() {
    let pid = 58608;

    println!("Executing: tasklist.exe /FI \"PID eq {}\"", pid);
    println!("{}", "=".repeat(80));

    let output = Command::new("tasklist.exe")
        .args(&["/FI", &format!("PID eq {}", pid)])
        .output()
        .expect("Failed to execute tasklist");

    println!("Exit status: {}", output.status);
    println!("Success: {}", output.status.success());
    println!();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("STDOUT ({} bytes):", output.stdout.len());
    println!("{}", stdout);
    println!();

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDERR ({} bytes):", output.stderr.len());
    println!("{}", stderr);
    println!();

    // Parse it
    println!("{}", "=".repeat(80));
    println!("Parsing:");
    for (i, line) in stdout.lines().enumerate() {
        println!("Line {}: '{}'", i, line);
        if i >= 2 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            println!("  Parts: {:?}", parts);
            if !parts.is_empty() {
                println!("  parts[0] = '{}'", parts[0]);
            }
        }
    }
}
