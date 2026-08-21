//! Terminal lifecycle: raw mode + alternate screen, restored on every exit
//! path (clean exit, `?` early return, panic).
//!
//! `Guard` restores the terminal when dropped; `install_panic_hook` adds a
//! panic hook that logs the panic and restores the terminal before the
//! default panic output, so a mid-run panic never leaves the shell in raw
//! mode on the alternate screen.

use std::io;

/// Restore the terminal: leave the alternate screen and disable raw mode.
/// Best-effort — failures are swallowed because there is nothing sensible
/// left to do during cleanup.
pub fn restore() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);
}

/// Guard that restores the terminal on drop. Create it before initializing
/// the UI and keep it alive for the whole UI session.
pub struct Guard;

impl Guard {
    /// Enter raw mode and the alternate screen.
    pub fn enter() -> io::Result<Guard> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        Ok(Guard)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        restore();
    }
}

/// Install a panic hook that logs the panic and restores the terminal before
/// running the previously installed hook (by default: message + backtrace).
/// Install this before entering the terminal.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("deepdelve: panic — restoring terminal");
        restore();
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_is_best_effort_and_repeatable() {
        // No tty in the test harness: restore must not fail or panic, and
        // must be safe to call repeatedly (the panic hook and the Drop
        // guard both run on panic).
        restore();
        restore();
    }
}
