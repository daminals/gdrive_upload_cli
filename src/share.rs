// share function
// Daniel Kogan
// 02.25.2022

use std::process::{Command, Stdio, exit};

// share
pub fn share(shared: &str, base_dir_id: &str) {
    // colors
    let green = "\u{001b}[32m";
    let clear_format = "\u{001b}[0m";
    // function
    let share_to = unwrap_share(&shared);
    if !(shared == "") { // if share is not empty...
        for email in share_to {
            let share_cmd = format!("gdrive share --type user --role writer --email {} {}", email, &base_dir_id);
            let share_execute_cmd = Command::new("sh").arg("-c").arg(share_cmd).stdout(Stdio::piped()).output().unwrap();
            println!("Directory shared with: {}{}{}", &green, &email, &clear_format);
        }
    }
}

// unwrap share cli argument
fn unwrap_share(shared: &str) -> Vec<&str> {
    return shared.split(",").collect::<Vec<&str>>();
}
