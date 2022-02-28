// upload
// Daniel Kogan
// 02.01.2022

#![allow(unused)]

// import external
use clap::{Arg, Subcommand};
use clap::Command as cliCommand;
use std::{fmt, fs, env};
use std::io::Write;
use std::fs::metadata;
use std::path::Path;
use std::process::{Command, Stdio, exit};
use std::collections::HashMap;
extern crate dotenv;

// import from self
mod append;
mod unwrap;
mod share;
//use unwrap::structs;
use unwrap::{GdriveQuery, FileId};

// colors for readable outputs
static RED: &str = "\u{001b}[31m";
static GREEN: &str = "\u{001b}[32m";
static YELLOW: &str = "\u{001b}[33m";
static CLEAR_FORMAT: &str = "\u{001b}[0m";
static UNDERLINE: &str = "\u{001b}[4m";

// create a hashmap of course names to folder id's
// if not in hash map, use whatever user entered (could be folder ID)
fn class_hashmap() -> std::collections::HashMap<&'static str, std::string::String> {
    let csehashmap = HashMap::from([
        ("160", env::var("UPLOAD160").unwrap()),
        ("projects", env::var("UPLOADp").unwrap()),
    ]);
    return csehashmap;
}

// TODO: add a feature that auto deletes files that are on drive 
// but not the uploading directory
fn main() {
    // dotenv
    let upload_tool_dotenv = env::var("UPLOADdotenv").unwrap();
    let dotenv_error = format!("{}Encountered an error reading {}.env{}", RED, UNDERLINE, CLEAR_FORMAT);
    dotenv::from_path(upload_tool_dotenv).expect("Encountered an error reading .env");
    // load dotenv required to access the gdrive course hashmap for this project

    let matches = cliCommand::new("Homework Uploader")
        .version("0.1.3")
        .author("Daniel Kogan")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .about("Uploads my directories to my google drive")
        .subcommand(
            cliCommand::new("u")
            .about("Uploads directories to gdrive")
            // arguments required in order to
            // execute the directory upload command
            .arg(
                Arg::new("course") // which course to upload to
                    .short('c') // will check hashmap for drive folder id
                    .long("course")
                    .takes_value(true)
                    .help("Stony Brook Course Number"),
            )
            .arg(
                Arg::new("directory") // which directory to upload
                    .short('d') // default is .
                    .long("dir")
                    .takes_value(true)
                    .help("What directory should I upload?"),
            ).arg(
                Arg::new("share") // share
                .short('s')
                .long("share")
                .takes_value(true)
                .help("Add emails to share directory to, seperate by comma")
            )
        )
        .subcommand(
            cliCommand::new("add")
            .about("Appends new environmental variables to program")
            .arg(
                Arg::new("key") // add key
                    .short('a')
                    .long("key")
                    .takes_value(true)
                    .help("Add new env var to tool")
            )
            .arg(
                Arg::new("value") // value
                    .short('v')
                    .long("value")
                    .takes_value(true)
                    .help("Add value to new env name")
            )
        )
        .get_matches();

    ctrlc::set_handler(move || { // exit program early
        println!("{} Exiting Program...{}", RED, CLEAR_FORMAT);
        exit(0); // actually exit program
    })
    .expect("Error setting Ctrl-C handler");

    // match the different subcommands to see if uploading
    // or if appending new envs
    if !(matches.subcommand_matches("u").is_none()) {
        // get subcommand matches
        let upload_matches = matches.subcommand_matches("u").unwrap();
        // unwrap
        let course = unwrap::unwrap_keys(upload_matches.value_of("course"), true);
        let dir = upload_matches.value_of("directory").unwrap_or(".");
        let share = unwrap::unwrap_keys(upload_matches.value_of("share"), false);
        // check name of current directory
        let get_basedir_cmd = format!("echo $(basename \"$PWD\")");
        let get_basedir_spawn = Command::new("sh").arg("-c").arg(get_basedir_cmd).stdout(Stdio::piped()).output().unwrap();
        let mut get_basedir_str = String::from_utf8(get_basedir_spawn.stdout).unwrap();
        // run command
        command_line(&course, &dir, &share, true, get_basedir_str);
    } else if !(matches.subcommand_matches("add").is_none()) {
        // get subcommand matches
        let append_matches = matches.subcommand_matches("add").unwrap();
        // unwrap
        let key = unwrap::unwrap_keys(append_matches.value_of("key"), true);
        let value = unwrap::unwrap_keys(append_matches.value_of("value"), true);
        // run command
        append::append_envs(key, value);
    } else {
        let error_msg = format!("{}Error! No argument detected! {}", RED, CLEAR_FORMAT);
        panic!("{}", error_msg);
    }
}
// upload function
fn command_line(course: &str, dir: &str, share: &str, base_case: bool, base_dir: String) {
    // course: look up in hashmap if coursename matches a class ID
    // dir: which directory to upload
    let paths = fs::read_dir(dir).unwrap();
    let cse_folder_id = return_parent(course);
    // dot_driveignore
    let dot_driveignore = unwrap::unwrap_dot_driveignore();
    let dot_driveignore = dot_driveignore.lines().collect::<Vec<_>>();
    // return the proper gdrive query struct
    unwrap::is_trashed(&base_dir, *&base_case); // check if trashed before setting struct to 
    //                                   preserve result struct integrity
    let result_struct = GdriveQuery::query(&cse_folder_id, &base_dir);
    if result_struct.update && unwrap::is_not_trashed(&base_dir, false) {
        print!("{}Updating Google Folder: {}  ⏳{}\n", YELLOW, &base_dir.trim(), CLEAR_FORMAT);
    } else {
        print!("{}Uploading Google Folder: {}  ⏳{}\n", YELLOW, &base_dir.trim(), CLEAR_FORMAT);
    }
    // make gdrive dir to upload to here
    let base_dir_id = return_base_directory(&result_struct, &cse_folder_id, &base_dir, base_case);
    // shares base drive with the specified users...
    share::share(&share, &base_dir_id);

    for path in paths {
        let readable_path = path.as_ref().unwrap().path().display().to_string();
        // write some tests

        // is this path a .git
        if readable_path.contains(".git") {
            continue;
        }
        // is this path a .class 
        if readable_path.contains(".class") {
            continue;
        }
        // is this path my dot driveignore
        if readable_path.contains(".driveignore") {
            continue;
        }
        // is this path in my dot driveignore
        if dot_driveignore.contains(&&readable_path.to_owned()[..]) {
            println!("Ignoring {}...", readable_path);
            continue;
        }

        // is this path directory?
        let is_directory = metadata(readable_path).unwrap();
        if is_directory.is_dir() {
            // full path is what I use for recursing through this directory (its location)
            let full_path = &path.as_ref().unwrap().path().display().to_string();
            // short path will be this directory's name on google drive
            // take the last / so its the name of the current folder
            let short_path = &path.as_ref().unwrap().path().display().to_string();
            let short_path = short_path.split("/").last().unwrap();

            // these functions are for checking if subfolder already exists
            //println!("Querying for {}", short_path);
            let sub_result_struct = GdriveQuery::query( &base_dir_id, &String::from(short_path));

            // update or upload
            if sub_result_struct.update && unwrap::is_not_trashed(&base_dir, false) {
                command_line(&base_dir_id, full_path, "", false, String::from(format!("{}\n",short_path)));
            } else {
                // upload new folder to the created gdrive folder (not course folder)
                // give folder name as dir name
                let create_cmd = format!("gdrive mkdir --parent {} {}", &base_dir_id, short_path);
                let subdir = Command::new("sh").arg("-c").arg(create_cmd).stdout(Stdio::piped()).output().unwrap();
                assert!(subdir.status.success()); // make sure it worked !!
                let mut subdir_name_full = String::from_utf8(subdir.stdout).unwrap();
                
                // take the new directory ID to upload to it, use full path as location
                let subdir_id = unwrap::unwrap_new_dir(subdir_name_full);
                command_line(&subdir_id, full_path, "", false, String::from(&base_dir));
            }
            continue;
        }

        // if it finally meets all conditions, upload or update the current file
        // find the file id. pipe in the id of the current drive directory in order to query it
        let path_id = FileId::get(&result_struct, &result_struct.id, &path);

        let cmd = unwrap::return_upload_or_update_cmd(&path_id, &base_dir_id, &path);
        // running this while saving the output auto-terminates process when done
        
        // error message formatting  
        let error_message = format!("{} error sending file to gdrive {}", RED, CLEAR_FORMAT);
        // upload / update command
        let output = Command::new("sh").arg("-c").arg(&cmd).stdout(Stdio::piped()).output().expect(&error_message);
        assert!(output.status.success()); // make sure it worked !!
        print!("{}", String::from_utf8(output.stdout).unwrap());
    }
    //end process
    println!("{}Processes completed ✅{}", GREEN, CLEAR_FORMAT);
    exit(0);
}
// program-specific unwrappers 
// return the new parent directory when creating a google folder
fn return_parent(fname: &str) -> String {
    let cse_hashmap = class_hashmap();
    if cse_hashmap.contains_key(fname) {
        let cse_folder_id = cse_hashmap.get(fname);
        return cse_folder_id.unwrap().to_string();
    } else {
        let cse_folder_id = fname.to_owned();
        return cse_folder_id;
    }
}
// return the id of the current gdrive base directory 
fn return_base_directory(gstruct: &GdriveQuery, cse_folder_id: &String, get_basedir_str: &String, base_case: bool) -> String {
    if !base_case {
        return cse_folder_id.to_owned();
    }
    if gstruct.update  && unwrap::is_not_trashed(&cse_folder_id, false){
        return gstruct.id.to_owned();
    } else {
        let create_base_dir = format!("gdrive mkdir --parent {} {}", cse_folder_id, get_basedir_str); // NOTE: second var has trailing whitespace -- be careful when updating code
        let dir = Command::new("sh").arg("-c").arg(create_base_dir).stdout(Stdio::piped()).output().unwrap();
        let mut dir_name_full = String::from_utf8(dir.stdout).unwrap();
        return unwrap::unwrap_new_dir(dir_name_full);
    }
}