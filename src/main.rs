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

    //println!("{:?}, {:?}, {:?}", course, folder_name, dir);

    ctrlc::set_handler(move || {
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

    // TODO: check if the folder already exists
    // this is the gdrive command
    //gdrive list --query " '14ejUWurUXx5bS3YFs7x3DGxFzSh_dGln' in parents "
    // will be pain in the booty to update so i will do it later

    // make gdrive dir to upload to here
    let create_base_dir = format!("uploadname=$(basename \"$PWD\") && gdrive mkdir --parent {} $uploadname", &cse_folder_id);
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
            let mut subdir_name_full = String::from_utf8(subdir.stdout).unwrap();
            print!("{}", subdir_name_full);
            // take the new directory ID to upload to it, use full path as location
            let subdir_id = unwrap_new_dir(subdir_name_full);
            command_line(&subdir_id, full_path);
            continue;
        }

        // if it finally meets all conditions, upload the current file
        let cmd = format!("gdrive upload --parent {} {}", &base_dir_id, path.as_ref().unwrap().path().display());
        let output = Command::new("sh").arg("-c").arg(cmd).stdout(Stdio::piped()).output().expect("An error as occured");
        print!("{}", String::from_utf8(output.stdout).unwrap());
        assert!(output.status.success());
    }
    // TODO: Quit when finished
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
