// addendum function
// Daniel Kogan
// 02.25.2022

use std::fs::File;
use std::io::Write;
use std::{env, fs};
use std::process::{Command, Stdio, exit};

static APPEND_NUM: usize = 33;

pub fn append_envs(key: &str, value: &str) {
    // colors
    let red = "\u{001b}[31m";
    let green = "\u{001b}[32m";
    let clear_format = "\u{001b}[0m";
    //config
    let config_file = env::var("UPLOADdotenv").unwrap();
    let config_addendum = format!("{}={}", key, value);
    //commnds
    let append_config_cmd = format!("sudo echo \"{}\" >> {}", config_addendum, config_file);
    let spawn_append_cmd = Command::new("sh").arg("-c").arg(append_config_cmd).stdout(Stdio::piped()).output().unwrap();
    //let output = String::from_utf8(spawn_append_cmd.stdout).unwrap();
    // update hashmap
    let rust_addendum = format!("        (\"{}\", env::var(\"UPLOAD{}\").unwrap()),", key, key);
    let this_dir = env::var("rs_file").unwrap();
    let this_file = format!("{}", &this_dir);
    let error_message = format!("{}Unable to read {} {}", &red, &this_file, &clear_format);
    let contents = fs::read_to_string(&this_file)
        .expect(&error_message);
    let mut content_new_lines = contents.lines().collect::<Vec<_>>();
    // format the new_line
    let new_line = format!("{}\n{}",content_new_lines[APPEND_NUM], rust_addendum); // edit the hashmap to add the new appended variable
    content_new_lines[APPEND_NUM] = &new_line[..];
    
    let mut write_file = File::create(this_file).expect("Error opening file");
    for line in content_new_lines {
        writeln!(&write_file, "{}", line).unwrap();
    }
    let update_program_cmd = format!("cd {} && ./update ", &this_dir);
    let run_update = Command::new("sh").arg("-c").arg(update_program_cmd).stdout(Stdio::piped()).output().unwrap();
    println!("{}Processes completed ✅{}", &green, &clear_format);
}
