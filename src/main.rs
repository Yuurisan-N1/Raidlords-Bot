#![allow(dead_code)]

mod army;
mod champ;
mod chest;
mod cli;
mod entry;
mod logika;
mod net;
mod quests;
mod raid;
mod run;
mod state;
mod store;
mod support;
mod treasury;
mod village;
mod wallet;
mod war;

use std::process::ExitCode;

fn main() -> ExitCode {
    support::banner::print_banner();
    support::log::install();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().cloned().unwrap_or_else(|| "run".to_string());

    if mode != "selftest" && mode != "verify" {
        stop::install();
    }

    let code = cli::dispatch(&args);
    ExitCode::from(code as u8)
}

#[cfg(unix)]
mod stop {
    extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }

    extern "C" fn on_signal(_signum: i32) {
        crate::support::log::blank();
        crate::support::log::lr("Script stopped by user");
        std::process::exit(0);
    }

    pub fn install() {
        unsafe {
            let handler: extern "C" fn(i32) = on_signal;
            signal(2, handler as usize);
        }
    }
}

#[cfg(windows)]
mod stop {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCtrlHandler(handler: Option<extern "system" fn(u32) -> i32>, add: i32) -> i32;
    }

    extern "system" fn on_console(_kind: u32) -> i32 {
        crate::support::log::blank();
        crate::support::log::lr("Script stopped by user");
        std::process::exit(0);
    }

    pub fn install() {
        unsafe {
            SetConsoleCtrlHandler(Some(on_console), 1);
        }
    }
}

#[cfg(not(any(unix, windows)))]
mod stop {
    pub fn install() {}
}
