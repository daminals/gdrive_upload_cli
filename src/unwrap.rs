// unwrappers
// helper functions for CLI
// Daniel Kogan
// 02.25.2022

use std::path::Path;
use clap::{App, Arg};
use std::{fmt, fs, env};
use std::io::Write;
use std::fs::metadata;
use std::process::{Command, Stdio, exit};
use std::collections::HashMap;


#[derive(Debug)]
pub struct GdriveQuery {
    pub id: String,
    pub name: String,
    pub gtype: String,
    pub dob: String,
    pub age: String,
    pub update: bool
}

impl GdriveQuery {
    // is gdrive query a directory?
    pub fn is_dir(&self) -> bool {
        return self.gtype == "dir";
    }
    // check if the gdrive query returns none
    pub fn is_none(query: &String) -> GdriveQuery {
        if !query.is_empty() {
            let result_struct = GdriveQuery::unwrap(query);
            return result_struct;
        } else {
            let result_struct = GdriveQuery { id: "".to_string(), name: "".to_string(), gtype: "".to_string(), dob: "".to_string(), age: "".to_string(), update: false };
            return result_struct;
        }
    }
    // unwrap the results of a gdrive query into a struct
    pub fn unwrap(result: &String) -> GdriveQuery {
        let result_vector = result.split_whitespace().collect::<Vec<&str>>();
        return GdriveQuery{ id: result_vector[0].to_string(), name: result_vector[1].to_string(), gtype: result_vector[2].to_string(), dob: result_vector[3].to_string(), age: result_vector[4].to_string(), update: true };
    }
    // search for folders in base directory
    pub fn query(folder_id: &String, search_string: &String) -> GdriveQuery {
        //println!("{}", folder_id);
        let check_gdrive_cmd = format!("gdrive list --query \" \'{}\' in parents \"", folder_id);
        let check_gdrive = Command::new("sh").arg("-c").arg(check_gdrive_cmd).stdout(Stdio::piped()).output().unwrap();
        let mut gdrive_cmd_output = String::from_utf8(check_gdrive.stdout).unwrap();
        //("I am looking for {}", search_string);
        //println!("All Possible Results are: {}", gdrive_cmd_output);
        //println!("{}", &gdrive_cmd_output);
        let query_result = unwrap_gdrive_query(gdrive_cmd_output, search_string);
        //println!("The result of my query is: {}", query_result);
        return GdriveQuery::is_none(&query_result);
    }
}

// create special FileId struct to organize related functions
pub struct FileId {
    id: String
}

impl FileId {
    // ~~~ private implementations (not needed for external functions) ~~~
    // safely unwrap the drive file's id
    fn unwrap(&self) -> FileId {
        if self.id.is_empty() {
            return FileId{id: "".to_owned()};
        } else {
            // skirt around the borrow checker
            return FileId { id: convert_string_ref(&self.id) };
        }
    }
    // return the current file id 
    fn return_file_id(gstruct: &GdriveQuery, folder_id: &String, path: &std::result::Result<std::fs::DirEntry, std::io::Error>) -> FileId {
        // short path will be this directory's name on google drive
        // take the last / so its the name of the current folder
        let short_path = path.as_ref().unwrap().path().display().to_string();
        let short_path = short_path.split("/").last().unwrap();
        if gstruct.update {
            //println!("I am Querying: {}", short_path);
            let file_query = GdriveQuery::query(folder_id, &String::from(short_path));
            if file_query.update{
                return FileId { id: file_query.id };
            } else {
                return FileId { id: String::from("") };
            } 
        } else {
            return FileId { id: String::from("") };
        } 
    }
}

impl FileId {
    // ~~~ public implementations ~~~
    // get the file id as a string from parameters
    pub fn get(gstruct: &GdriveQuery, folder_id: &String, path: &std::result::Result<std::fs::DirEntry, std::io::Error>) -> String {
        let file_id = FileId::return_file_id(gstruct, folder_id, path);
        return file_id.unwrap().to_str();
    }
    // convert file_id to string
    pub fn to_str (&self) -> String{
        return convert_string_ref(&self.id);
    }
}

// turn string reference to new string
fn convert_string_ref(borrowed_string: &str) -> String { 
    // skirt around the borrow checker
    return String::from(borrowed_string);
}

// read cli arguments
pub fn unwrap_keys(keyword: Option<&str>, mandatory: bool) -> &str {
    // if no folder name, set it to folder name of where command is run from
    if !keyword.is_none() {
        return keyword.unwrap();
    } else {
        if (mandatory) {
            panic!("No keyword provided")
        } else {
            return "";
        }
    }
}

// strip directory string so only the gdrive ID is left
pub fn unwrap_new_dir(mut directory: String) -> std::string::String {
    let mut i = 0;
    while i < 8 {
        directory.pop();
        i+=1;
    } 
    let dir_id = directory[10..].to_owned();
    return dir_id;
}

// read the dot driveignore file. Return "" if non-existent
pub fn unwrap_dot_driveignore() -> std::string::String {
    let contents;
    if Path::new(".driveignore").exists() {
        contents = fs::read_to_string(".driveignore").expect("Something went wrong reading the file");
    } else {
        contents = String::from("");
    }
    return contents;
}

// unwrap gdrive query output  
pub fn unwrap_gdrive_query(cmd_output: String, search_string: &String) -> String {
    let mut split_output_lines = cmd_output.lines().skip(1).collect::<Vec<_>>();
    for item in split_output_lines {
        // do this so that it wont flag a search that our search is a substring of
        // exact terms only
        let search_term = format!("{} ", search_string.trim_end()); 
        if item.contains(&search_term) {
                // strip the string to just the id from this
                return String::from(item);
        }
    }
    return String::from("");
}

// return the command for uploading/updating the file
pub fn return_upload_or_update_cmd(file_id: &String, parent_id: &String, path: &std::result::Result<std::fs::DirEntry, std::io::Error>) -> std::string::String {
    if !file_id.is_empty() && !is_trashed(&file_id, false) {
        //println!("{}", file_id);
        return format!("gdrive update {} {}", file_id, path.as_ref().unwrap().path().display());
    } else {
        return format!("gdrive upload --parent {} {}", parent_id, path.as_ref().unwrap().path().display());
    }
}

// ~~~ GDRIVE TRASH RELATED COMMANDS ~~~

// query grdrive trash for search string. Return true if it is there
pub fn is_trashed(search_string: &String, prompt: bool) -> bool {
    let trash_query = gdrive_trash_query(&search_string);
    if !trash_query.is_empty() && prompt {
        // colors for readable output
        let gray_col = "\u{001b}[90m";
        let yellow = "\u{001b}[33m";
        let clear_format = "\u{001b}[0m";
        // trash function
        let user_readable_output = format!("{}{}{}", yellow, search_string.trim_end(),clear_format);
        print!("It seems that {} is in your drive trash. Delete? {}(Y/n){}  ", user_readable_output, gray_col, clear_format);
        std::io::stdout().flush().unwrap();
        if (return_user_input().to_uppercase() == String::from("Y")) {
            let trashed_file_id = GdriveQuery::unwrap(&trash_query); // we want to unwrap
            // the trashed file into a usable format for deletion
            let delete_trash_cmd = format!("gdrive delete -r {}", trashed_file_id.id);
            let delete_trash_stdout = Command::new("sh").arg("-c").arg(delete_trash_cmd)
                .stdout(Stdio::piped()).output().unwrap();
            println!("{}", String::from_utf8(delete_trash_stdout.stdout).unwrap());
            is_trashed(&search_string, true);
        } 
    } // even if file is deleted, we dont want to update file, but upload a new version
    return !trash_query.is_empty(); // if the query returned none, it is not in trash
}

pub fn is_not_trashed(search_string: &String, prompt: bool) -> bool {
    return !is_trashed(search_string, prompt);
}

// return the result from the trash query
// when this program runs it will output all trashed files in drive
// and return the first match with the name of the search string
pub fn gdrive_trash_query(search_string: &String) -> String {
    let query_trash_cmd = "gdrive list -q \"\"trashed\" = true\"";
    let trash_stdout = Command::new("sh").arg("-c").arg(&query_trash_cmd)
    .stdout(Stdio::piped()).output().unwrap();
    let mut trash = String::from_utf8(trash_stdout.stdout).unwrap();
    let trash_query = unwrap_gdrive_query(trash, search_string);
    return trash_query;
}

// ~~~ USER INPUT ~~~

// prompt user
pub fn return_user_input() -> String {
    let mut user_input = String::new();
    std::io::stdin()
    .read_line(&mut user_input)
    .unwrap();
    return user_input.trim().to_string() // disregard the newline character from end
}
