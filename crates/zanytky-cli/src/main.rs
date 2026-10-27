use std::io::{BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut stdout = std::io::stdout();
    let no_command = args.is_empty();
    let outcome = zanytky_cli::run(&args, &mut stdout);
    if let Err(message) = &outcome {
        eprintln!("Помилка: {message}");
    }
    let _ = stdout.flush();
    if no_command {
        // Started by a double click: keep the window open so the help can be read.
        println!();
        println!("Це програма командного рядка: запускайте її з термінала (PowerShell) з командою.");
        println!("Натисніть Enter, щоб закрити це вікно.");
        let _ = stdout.flush();
        let mut line = String::new();
        let _ = std::io::stdin().lock().read_line(&mut line);
    }
    if outcome.is_err() {
        std::process::exit(1);
    }
}
