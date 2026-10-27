use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut stdout = std::io::stdout();
    match zanytky_cli::run(&args, &mut stdout) {
        Ok(()) => {
            let _ = stdout.flush();
        }
        Err(message) => {
            eprintln!("Помилка: {message}");
            std::process::exit(1);
        }
    }
}
