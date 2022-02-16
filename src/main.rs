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
#[derive(Debug)]
struct GdriveQuery {
    id: String,
    name: String,
    gtype: String,
    dob: String,
    age: String,
    update: bool
}

// TODO: add a feature that auto deletes files that are on drive 
// but not the uploading directory
// TODO: if there is a folder in the trash of the same name 
// as a folder that can be updated, it will upload a new one instead 
// of updating the not-trashed folder
fn main() {
    let matches = App::new("Homework Uploader")
        .version("0.1.2")
        .author("Daniel Kogan")
        .about("Uploads my directories to my google drive")
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
        )
        .arg(
            Arg::new("key") // add key
                .short('a')
                .long("add")
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
        .arg(
            Arg::new("share") // share
            .short('s')
            .long("share")
            .takes_value(true)
            .help("Add emails to share directory to, seperate by comma")
        )
        .get_matches();

    ctrlc::set_handler(move || { // exit program early
        let red = "\u{001b}[31m";
        let clear_format = "\u{001b}[0m";    
        println!("{}Exiting Program...{}", red, clear_format);
    })
    .expect("Error setting Ctrl-C handler");

    if (check_uploading(matches.value_of("course"), matches.value_of("key"), matches.value_of("value"))) {
        // if uploading, do this
        let course = unwrap_keys(matches.value_of("course"), false, true);
        let dir = unwrap_keys(matches.value_of("directory"), true, false);
        let share = unwrap_keys(matches.value_of("share"), false, false);
        //println!("{:?}, {:?}", course, dir);
    
        // check name of current directory
        let get_basedir_cmd = format!("echo $(basename \"$PWD\")");
        let get_basedir_spawn = Command::new("sh").arg("-c").arg(get_basedir_cmd).stdout(Stdio::piped()).output().unwrap();
        let mut get_basedir_str = String::from_utf8(get_basedir_spawn.stdout).unwrap();
    
        command_line(&course, &dir, &share, true, get_basedir_str);
    } else { // if not uploading, append 
        let key = unwrap_keys(matches.value_of("a"), false, true);
        let value = unwrap_keys(matches.value_of("v"), false, true);

        append_envs(key, value);
    }
}
// upload function
fn command_line(course: &str, dir: &str, share: &str, base_case: bool, base_dir: String) {
    // colors 
    let yellow = "\u{001b}[33m";
    let green = "\u{001b}[32m";
    let clear_format = "\u{001b}[0m";
    // course: look up in hashmap if coursename matches a class ID
    // dir: which directory to upload
    let paths = fs::read_dir(dir).unwrap();
    let cse_folder_id = return_parent(course);
    let share_to = unwrap_share(&share);
    // dot_driveignore
    let dot_driveignore = unwrap_dot_driveignore();
    let dot_driveignore = dot_driveignore.lines().collect::<Vec<_>>();
    // return the proper gdrive query struct
    let result_struct = query_gdrive(&cse_folder_id, &base_dir);
    if result_struct.update && !is_trashed(&base_dir) {
        print!("{}Updating Google Folder: {}  ⏳{}\n", &yellow, &base_dir.trim(), &clear_format);
    } else {
        print!("{}Uploading Google Folder: {}  ⏳{}\n", &yellow, &base_dir.trim(), &clear_format);
    }
    // make gdrive dir to upload to here
    let base_dir_id = return_base_directory(&result_struct, &cse_folder_id, &base_dir, base_case);
    // shares base drive with the specified users...
    if !(share == "") { // if share is not empty...
        for email in share_to {
            let share_cmd = format!("gdrive share --type user --role writer --email {} {}", email, &base_dir_id);
            let share_execute_cmd = Command::new("sh").arg("-c").arg(share_cmd).stdout(Stdio::piped()).output().unwrap();
            println!("Directory shared with: {}{}{}", &green, &email, &clear_format);
        }
    }
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
            let sub_result_struct = query_gdrive( &base_dir_id, &String::from(short_path));

            // update or upload
            if sub_result_struct.update && !is_trashed(&base_dir) {
                command_line(&base_dir_id, full_path, "", false, String::from(format!("{}\n",short_path)));
            } else {
                // upload new folder to the created gdrive folder (not course folder)
                // give folder name as dir name
                let create_cmd = format!("gdrive mkdir --parent {} {}", &base_dir_id, short_path);
                let subdir = Command::new("sh").arg("-c").arg(create_cmd).stdout(Stdio::piped()).output().unwrap();
                assert!(subdir.status.success()); // make sure it worked !!
                let mut subdir_name_full = String::from_utf8(subdir.stdout).unwrap();
                print!("{}", subdir_name_full);
                
                // take the new directory ID to upload to it, use full path as location
                let subdir_id = unwrap_new_dir(subdir_name_full);
                command_line(&subdir_id, full_path, "", false, String::from(&base_dir));
            }
            continue;
        }

        // if it finally meets all conditions, upload or update the current file
        // find the file id. pipe in the id of the current drive directory in order to query it
        let file_id = return_file_id(&result_struct, &result_struct.id, &path);
        let path_id = unwrap_file_id(&file_id);
        //println!("{}, {}", file_id, path_id);
        //println!("{:?}", result_struct);

        let cmd = return_upload_or_update_cmd(&path_id, &base_dir_id, &path);
        // running this while saving the output auto-terminates process when done
        let output = Command::new("sh").arg("-c").arg(cmd).stdout(Stdio::piped()).output().expect("An error as occured");
        assert!(output.status.success()); // make sure it worked !!
        print!("{}", String::from_utf8(output.stdout).unwrap());
    }
    //end process
    println!("{}Processes completed ✅{}", &green, &clear_format);
    exit(0);
}
fn query_gdrive(folder_id: &String, search_string: &String) -> GdriveQuery {
    let check_gdrive_cmd = format!("gdrive list --query \" \'{}\' in parents \"", folder_id);
    let check_gdrive = Command::new("sh").arg("-c").arg(check_gdrive_cmd).stdout(Stdio::piped()).output().unwrap();
    let mut gdrive_cmd_output = String::from_utf8(check_gdrive.stdout).unwrap();
    //("I am looking for {}", search_string);
    //println!("All Possible Results are: {}", gdrive_cmd_output);
    //println!("{}", &gdrive_cmd_output);
    let query_result = unwrap_gdrive_query(gdrive_cmd_output, search_string);
    //println!("The result of my query is: {}", query_result);
    return check_gdrive_query_is_none(&query_result);
}
// unwrappers 
// read cli arguments
fn unwrap_keys(keyword: Option<&str>, dir: bool, mandatory: bool) -> &str {
    // if no folder name, set it to folder name of where command is run from
    if !keyword.is_none() {
        return keyword.unwrap();
    } else {
        if dir {
            return ".";
        } else if (mandatory) {
            panic!("No keyword provided")
        } else {
            return "";
        }
    }
}
// determine if the program should be uploading new files or updating old ones
fn check_uploading(course: Option<&str>, add: Option<&str>, value: Option<&str>) -> bool {
    if (add.is_none() != value.is_none()) {
        panic!("Var name and value belong together shawty 💔");
    }
    if (course.is_none() && !add.is_none()) {
        return false;
    } else if (add.is_none() && !course.is_none()) {
        return true;
    } else if (add.is_none() && course.is_none()) {
        panic!("No keywords provided");
    } else {
        panic!("Too many arguments");
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
// unwrap share cli argument
fn unwrap_share(share: &str) -> Vec<&str> {
    return share.split(",").collect::<Vec<&str>>();
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
// safely unwrap the drive file's id
fn unwrap_file_id(file_id: &String) -> String {
    if file_id.is_empty() {
        return "".to_owned();
    } else {
        return file_id.to_owned();
    }
}
// check if the gdrive query returns none
fn check_gdrive_query_is_none(query: &String) -> GdriveQuery {
    if !query.is_empty() {
        let result_struct = unwrap_gdrive_query_results(query);
        return result_struct;
    } else {
        let result_struct = GdriveQuery { id: "".to_string(), name: "".to_string(), gtype: "".to_string(), dob: "".to_string(), age: "".to_string(), update: false };
        return result_struct;
    }
}
// is gdrive query result a directory?
fn gdrive_query_is_dir(result: GdriveQuery) -> bool {
    return result.gtype == "dir";
}
// unwrap the results of a gdrive query into a struct
fn unwrap_gdrive_query_results(result: &String) -> GdriveQuery {
    let result_vector = result.split_whitespace().collect::<Vec<&str>>();
    return GdriveQuery{ id: result_vector[0].to_string(), name: result_vector[1].to_string(), gtype: result_vector[2].to_string(), dob: result_vector[3].to_string(), age: result_vector[4].to_string(), update: true };
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
// return the id of the current gdrive base directory 
fn return_base_directory(gstruct: &GdriveQuery, cse_folder_id: &String, get_basedir_str: &String, base_case: bool) -> std::string::String {
    if !base_case {
        return cse_folder_id.to_owned();
    }
    if gstruct.update  && !is_trashed(&cse_folder_id){
        return gstruct.id.to_owned();
    } else {
        let create_base_dir = format!("gdrive mkdir --parent {} {}", cse_folder_id, get_basedir_str); // NOTE: second var has trailing whitespace -- be careful when updating code
        let dir = Command::new("sh").arg("-c").arg(create_base_dir).stdout(Stdio::piped()).output().unwrap();
        let mut dir_name_full = String::from_utf8(dir.stdout).unwrap();
        return unwrap_new_dir(dir_name_full);
    }
}
// return the command for uploading/updating the file
fn return_upload_or_update_cmd(file_id: &String, parent_id: &String, path: &std::result::Result<std::fs::DirEntry, std::io::Error>) -> std::string::String {
    if !file_id.is_empty() && !is_trashed(&file_id) {
        //println!("{}", file_id);
        return format!("gdrive update {} {}", file_id, path.as_ref().unwrap().path().display());
    } else {
        return format!("gdrive upload --parent {} {}", parent_id, path.as_ref().unwrap().path().display());
    }
}
// return the current file id 
fn return_file_id(gstruct: &GdriveQuery, folder_id: &String, path: &std::result::Result<std::fs::DirEntry, std::io::Error>) -> String{
    // short path will be this directory's name on google drive
    // take the last / so its the name of the current folder
    let short_path = path.as_ref().unwrap().path().display().to_string();
    let short_path = short_path.split("/").last().unwrap();
    //println!("{:?}", gstruct);

    if gstruct.update {
        //println!("I am Querying: {}", short_path);
        let file_query = query_gdrive(folder_id, &String::from(short_path));
        if file_query.update{
            return file_query.id;
        } else {
            return String::from("")
        } 
    } else {
        return String::from("")
    }
}
// query grdrive trash for search string. Return true if it is there
fn is_trashed(search_string: &String) -> bool {
    let query_trash_cmd = "gdrive list -q \"trashed\" = true";
    let trash_stdout = Command::new("sh").arg("-c").arg(query_trash_cmd)
        .stdout(Stdio::piped()).output().unwrap();
    let mut trash = String::from_utf8(trash_stdout.stdout).unwrap();
    let trash_query = unwrap_gdrive_query(trash, search_string);
    return trash_query.is_empty(); // if the query returned none, it is not in trash
}
// addendum function
use std::fs::File;
use std::io::Write;

fn append_envs(key: &str, value: &str) {
    // colors
    let red = "\u{001b}[31m";
    let green = "\u{001b}[32m";
    let clear_format = "\u{001b}[0m";
    //config
    let config_file = env::var("config_file").unwrap();
    let addendum = format!("        (\"{}\", env::var(\"UPLOAD{}\").unwrap()),", key, key);
    let config_addendum = format!("echo export UPLOAD{}={}", key, value);
    //commnds
    let append_config_cmd = format!("sudo echo \"{}\" >> {}", config_addendum, config_file);
    let spawn_append_cmd = Command::new("sh").arg("-c").arg(append_config_cmd).stdout(Stdio::piped()).output().unwrap();
    //let output = String::from_utf8(spawn_append_cmd.stdout).unwrap();
    // update hashmap
    let this_dir = env::var("rs_file").unwrap();
    let this_file = format!("{}/src/main.rs", &this_dir);
    let error_message = format!("{}Something went wrong reading the file{}", &red, &clear_format);
    let contents = fs::read_to_string(&this_file)
        .expect(&error_message);
    let mut content_new_lines = contents.lines().collect::<Vec<_>>();
    // format the new_line
    let new_line = format!("{}\n{}",content_new_lines[23], addendum); // edit the hashmap to add the new appended variable
    content_new_lines[23] = &new_line[..];
    
    let mut write_file = File::create(this_file).expect("Error opening file");
    for line in content_new_lines {
        writeln!(&write_file, "{}", line).unwrap();
    }

    let update_program_cmd = format!("cd {} && ./update ", &this_dir);
    let run_update = Command::new("sh").arg("-c").arg(update_program_cmd).stdout(Stdio::piped()).output().unwrap();
    println!("{}Processes completed ✅{}", &green, &clear_format);
}
