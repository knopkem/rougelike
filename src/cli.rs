//! Command-line parsing for the deepdelve binary (PLAN §8: `--seed`,
//! `--headless`, `--no-audio`).

/// Parsed command line for a run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Args {
    /// Explicit run seed; `None` means generate one from the wall clock.
    pub seed: Option<u64>,
    /// Run the game loop without the curses UI.
    pub headless: bool,
    /// Start with the audio engine disabled.
    pub no_audio: bool,
}

/// Parse command-line arguments (the program name is already stripped).
///
/// Returns `Ok(None)` when `-h`/`--help` is requested, `Ok(Some(args))` on
/// success, and `Err(message)` for unknown flags or a malformed `--seed`
/// value.
pub fn parse<I>(args: I) -> Result<Option<Args>, String>
where
    I: IntoIterator<Item = String>,
{
    let mut out = Args::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--seed" => {
                let value = iter.next().ok_or("--seed requires a value")?;
                out.seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("--seed: not a valid u64: {value}"))?,
                );
            }
            "--headless" => out.headless = true,
            "--no-audio" => out.no_audio = true,
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("unknown argument: {other:?}")),
        }
    }
    Ok(Some(out))
}

/// Generate a run seed from the wall clock, mixed with the pid so two runs
/// started in the same nanosecond cannot collide.
pub fn generated_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos ^ (std::process::id() as u64).rotate_left(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn no_flags_gives_defaults() {
        let args = parse(s(&[])).unwrap().unwrap();
        assert_eq!(
            args,
            Args {
                seed: None,
                headless: false,
                no_audio: false,
            }
        );
    }

    #[test]
    fn all_flags_parse() {
        let args = parse(s(&["--seed", "42", "--headless", "--no-audio"]))
            .unwrap()
            .unwrap();
        assert_eq!(args.seed, Some(42));
        assert!(args.headless);
        assert!(args.no_audio);
    }

    #[test]
    fn seed_value_must_be_u64() {
        assert!(parse(s(&["--seed", "not-a-number"])).is_err());
        assert!(parse(s(&["--seed", "-3"])).is_err());
        assert!(parse(s(&["--seed"])).is_err());
    }

    #[test]
    fn unknown_flag_is_an_error() {
        assert!(parse(s(&["--bogus"])).is_err());
    }

    #[test]
    fn help_short_circuits() {
        assert_eq!(parse(s(&["--help"])).unwrap(), None);
        assert_eq!(parse(s(&["-h", "--seed", "1"])).unwrap(), None);
    }
}
