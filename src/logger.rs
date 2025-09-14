use colored::Colorize;

pub fn section(name: &str) {
    println!("{}", name.bold());
    println!("{}", "-".repeat(25));
}

pub fn info(message: &str) {
    println!("{}  {}", "-".bright_black().bold(), message.bright_black());
}

pub fn completion(message: &str) {
    println!("{}  {}", "✓".green().bold(), message);
}

pub fn error(message: &str) {
    println!("\n{} {}", "error:".red().bold(), message);
}
