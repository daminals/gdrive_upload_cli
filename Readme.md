![github repo badge: Language](https://img.shields.io/badge/Language-Rust-181717?color=orange) ![github repo badge: Using](https://img.shields.io/badge/Using-gdrive-181717?color=blue)

# Upload

## Description

A commmand line tool for uploading homework coded on the dcloud server onto specific google drive course folders.

The program uses rust and integrates the gdrive cli in order to config and organize uploads in a desired format.

## What I learned

I learned how to create a sophisticated command line interface through rust, incorporating bash processes and the borrowing/ownership concept in rust.

## Features

- reads custom .driveignore file to prevent uploading unwanted files
- reads google drive to check if file already exists, if yes, updates it rather than create a new one
- organizes all uploads under a directory name to prevent clutter in gdrive
- recursion through nested directories ensures all files are uploaded
- hashmap-integrated environment variables for fast processing
- descriptive and informative CLI output so users can follow along