// upload
// Daniel Kogan
// 02.01.2022

#![allow(unused)]

use clap::{App, Arg};
use std::fmt;
use std::fs;
use std::fs::metadata;
use std::path::Path;
use std::process::{Command, Stdio, exit};
use std::env;
use std::collections::HashMap;

// create a hashmap of course names to folder id's
// if not in hash map, use whatever user entered (could be folder ID)
fn class_hashmap() -> std::collections::HashMap<&'static str, std::string::String> {
    let cse160 = env::var("UPLOAD160").unwrap();
    let projects = env::var("UPLOADp").unwrap();
    //println!("{}", cse160);
    let csehashmap = HashMap::from([
        ("160", env::var("UPLOAD160").unwrap()),
        ("projects", env::var("UPLOADp").unwrap()),
    ]);
    return csehashmap;
}

fn main() {
    let matches = App::new("Homework Uploader")
        .version("0.1.1")
        .author("Daniel Kogan")
        .about("Uploads my directories to my google drive")
        .arg(
            Arg::new("class")
                .short('c')
                .long("class")
                .takes_value(true)
                .help("Stony Brook Course Number"),
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .takes_value(true)
                .help("What directory should I upload?"),
        )
        .get_matches();

    let course = unwrap_keys(matches.value_of("class"), false);
    let dir = unwrap_keys(matches.value_of("directory"), true);
    //println!("{:?}, {:?}", course, dir);

    ctrlc::set_handler(move || { // exit program early
        println!("received Ctrl+C!");
    })
    .expect("Error setting Ctrl-C handler");

    command_line(&course, &dir);
}

fn command_line(course: &str, dir: &str) {
    // course: look up in hashmap if coursename matches a class ID
    // dir: which directory to upload
    let paths = fs::read_dir(dir).unwrap();
    let cse_folder_id = return_parent(course);

    // dot_driveignore
    let dot_driveignore = unwrap_dot_driveignore();
    let dot_driveignore = dot_driveignore.lines().collect::<Vec<_>>();

    // check name of current directory
    let get_basedir_cmd = format!("echo $(basename \"$PWD\")");
    let get_basedir_spawn = Command::new("sh").arg("-c").arg(get_basedir_cmd).stdout(Stdio::piped()).output().unwrap();
    let mut get_basedir_str = String::from_utf8(get_basedir_spawn.stdout).unwrap();

    // TODO: check if the folder already exists
    // this is the gdrive command
    //gdrive list --query " '14ejUWurUXx5bS3YFs7x3DGxFzSh_dGln' in parents "
    // will be pain in the booty to update so i will do it later
    
    let check_gdrive_cmd = format!("gdrive list --query \" \'{}\' in parents \"", &cse_folder_id);
    let check_gdrive = Command::new("sh").arg("-c").arg(check_gdrive_cmd).stdout(Stdio::piped()).output().unwrap();
    let mut gdrive_cmd_output = String::from_utf8(check_gdrive.stdout).unwrap();
    //println!("{}", &gdrive_cmd_output);
    unwrap_gdrive_query(gdrive_cmd_output, &get_basedir_str);

    exit(0);
    // make gdrive dir to upload to here
    let create_base_dir = format!("gdrive mkdir --parent {} {}", &cse_folder_id, &get_basedir_str); // NOTE: second var has trailing whitespace -- be careful when updating code
    let dir = Command::new("sh").arg("-c").arg(create_base_dir).stdout(Stdio::piped()).output().unwrap();
    let mut dir_name_full = String::from_utf8(dir.stdout).unwrap();
    let base_dir_id = unwrap_new_dir(dir_name_full);
    
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
            // upload new folder to the created gdrive folder (not course folder)
            // give folder name as dir name
            let create_cmd = format!("gdrive mkdir --parent {} {}", &base_dir_id, short_path);
            let subdir = Command::new("sh").arg("-c").arg(create_cmd).stdout(Stdio::piped()).output().unwrap();
            assert!(subdir.status.success()); // make sure it worked !!
            let mut subdir_name_full = String::from_utf8(subdir.stdout).unwrap();
            print!("{}", subdir_name_full);
            // take the new directory ID to upload to it, use full path as location
            let subdir_id = unwrap_new_dir(subdir_name_full);
            command_line(&subdir_id, full_path);
            continue;
        }

        // if it finally meets all conditions, upload the current file
        let cmd = format!("gdrive upload --parent {} {}", &base_dir_id, path.as_ref().unwrap().path().display());
        // running this while saving the output auto-terminates process when done
        let output = Command::new("sh").arg("-c").arg(cmd).stdout(Stdio::piped()).output().expect("An error as occured");
        assert!(output.status.success()); // make sure it worked !!
        print!("{}", String::from_utf8(output.stdout).unwrap());
    }
    //end process
    exit(0);
}
// unwrappers 
// read cli arguments
fn unwrap_keys(keyword: Option<&str>, dir: bool) -> &str {
    // if no folder name, set it to folder name of where command is run from
    if !keyword.is_none() {
        return keyword.unwrap();
    } else {
        if dir {
            return ".";
        }
        panic!("No keyword provided");
    }
}
// strip directory string so only the gdrive ID is left
fn unwrap_new_dir(mut directory: String) -> std::string::String {
    let mut i = 0;
    while i < 8 {
        directory.pop();
        i+=1;
    } 
    let dir_id = directory[10..].to_owned();
    return dir_id;
}
// read the dot driveignore file. Return "" if non-existent
fn unwrap_dot_driveignore() -> std::string::String {
    let contents;
    if Path::new(".driveignore").exists() {
        contents = fs::read_to_string(".driveignore").expect("Something went wrong reading the file");
    } else {
        contents = String::from("");
    }
    return contents;
}
// unwrap gdrive query output  
fn unwrap_gdrive_query(cmd_output: String, search_string: &String) -> String {
    let mut split_output_lines = cmd_output.lines().skip(1).collect::<Vec<_>>();
    for item in split_output_lines {
        //println!("{}", item);
        if item.contains(search_string.trim_end()) {
                // strip the string to just the id from this
                return String::from(item);
        }
    }
    return String::from("");
}
fn gdrive_query_is_dir(result: String) -> bool {
    return result.contains("dir");
}
fn unwrap_gdrive_query_results() {
    // still need to write this lol
    // ignore everything after the first " " to get gdrive id so i can update it
    // can i update folders? probably not; write a recursive function to find 
    // every item in the query and update it individually
    // if searched item not updating folder, upload it   
}
// return the new parent directory when creating a google folder
fn return_parent(fname: &str) -> std::string::String {
    let cse_hashmap = class_hashmap();
    if cse_hashmap.contains_key(fname) {
        let cse_folder_id = cse_hashmap.get(fname);
        return cse_folder_id.unwrap().to_string();
    } else {
        let cse_folder_id = fname.to_owned();
        return cse_folder_id;
    }
}
