//! Deep Delve — entry point.
//!
//! Parses command-line arguments, then hands off to the UI application loop.
//!
//! Arguments:
//! - `--seed <u64>`: use a fixed seed (reproducible runs).
//! - `--no-audio`: disable audio (also the default in headless/CI builds).

use deepdelve::ui::App;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: Option<u64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(s) => seed = Some(s),
                        Err(_) => {
                            eprintln!("error: --seed requires a u64 value");
                            std::process::exit(2);
                        }
                    }
                    i += 1;
                } else {
                    eprintln!("error: --seed requires a value");
                    std::process::exit(2);
                }
            }
            "--no-audio" => {
                // Audio is a compile-time feature; this flag is accepted for
                // forward compatibility. To truly disable audio, build with
                // `--no-default-features`.
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("error: unknown argument '{}'", other);
                print_help();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let mut app = App::new(seed);
    app.run();
}

fn print_help() {
    println!("Deep Delve — a console roguelike");
    println!();
    println!("Usage: deepdelve [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --seed <u64>   Use a fixed seed for a reproducible run");
    println!(
        "  --no-audio     Disable audio (accepted; build with --no-default-features to truly disable)"
    );
    println!("  -h, --help     Show this help");
    println!();
    println!("Controls:");
    println!("  Movement: arrows or h/j/k/l (diagonals: u/y/b/n)");
    println!("  Wait: . or space    Stairs: > (down) < (up)");
    println!("  Pickup: , or g    Inventory: i    Help: ?");
    println!("  Save & quit: S    Abort: Q");
}
