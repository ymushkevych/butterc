pub mod x86_64_linux_gnu;

use std::{env::consts::OS, process::exit}; 

pub fn determine_system() -> String {
    if OS == "linux" {
        if cfg!(target_arch = "x86_64") {
            return "x86_64_linux_gnu".to_owned();
        } else {
            exit(11);
        }

    } else {
      exit(11);
    }
}
