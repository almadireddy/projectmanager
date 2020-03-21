extern crate clap;
extern crate dialoguer;
extern crate yaml_rust;
extern crate dirs;

use clap::{Arg, App, SubCommand};
use dialoguer::{theme::ColorfulTheme, Select};
use yaml_rust::{YamlLoader, Yaml};
use std::fs::{File};
use std::io::{Read, Error, ErrorKind};
use std::vec::Vec;
use std::result::Result;
use std::process::Command;

fn main() {
    let matches = App::new("Projects")
        .version("0.1")
        .author("Aahlad Madireddy")
        .about("easy project management")
        .subcommand(SubCommand::with_name("open")
                .about("open a project")
                .arg(Arg::with_name("projectname")
                        .required(true)
                        .index(1)))
        .subcommand(SubCommand::with_name("list")
                .about("list project tracked with project manager"))
        .subcommand(SubCommand::with_name("add")
                .about("add project to track")
                .arg(Arg::with_name("project_name")
                        .required(true)
                        .index(1))
                .arg(Arg::with_name("project_path")
                        .required(true)
                        .index(2)))
        .get_matches();

    match matches.subcommand() {
        ("open", Some(sub_command)) => open_project(&sub_command), 
        ("list", Some(_subc)) => list_projects(), 
        ("add", Some(sub_command)) => add_project(&sub_command), 
        (&_, _) => println!("{}", matches.usage())
    }    
}

fn load_projects_from_data() -> Result<Yaml, Error> {
    let home_directory = dirs::home_dir();
    let mut config_path: std::path::PathBuf;
    let f: std::io::Result<File>;

    match home_directory {
        Some(home) => {
            config_path = home.clone();
            config_path.push(".projectman");
            f = File::open(config_path.clone());
        }, 
        None => {
            println!("Error getting home directory");
            return Err(Error::new(ErrorKind::Other, "failed to find home directory"));
        },
    }

    match f {
        Ok(mut contents) => {
            let mut s = String::new();
            contents.read_to_string(&mut s)?;
            let docs = YamlLoader::load_from_str(&s).unwrap();
            if docs.len() == 0 {
                return Err(Error::new(ErrorKind::Other, "No yaml in file, may be corrupted"))
            }
            return Ok(docs[0].clone())
        }, 
        Err(e) => {
            if let Some(raw_err) = e.raw_os_error() {
                match raw_err {
                    2 => {
                        println!("file doesn't exist, creating");
                        std::fs::File::create(config_path)?;
                        let v: Yaml = Yaml::from_str("");
                        return Ok(v)
                    },
                    _ => {
                        println!("Other error reading data file");
                        return Err(e)  
                    } 
                }
            } else {
                return Err(e)
            }
        } 
    }
}

fn list_projects() {
    let projects_yaml = load_projects_from_data();
    let projects: Yaml; 

    match projects_yaml {
        Ok(pr) => {
            projects = pr.clone();
            println!("{:?}", projects);
        },
        Err(r) => {
            println!("error: {}", r);
            return
        },
    }

    let mut hash = projects.clone().into_hash().unwrap();
    let mut keys: Vec<String> = Vec::new();

    for entry in hash.entries() {
        let key = entry.key().as_str().unwrap().to_owned();
        keys.push(key);
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select project")
        .default(0)
        .items(keys.as_slice())
        .interact()
        .unwrap();

    println!("selection: {}", selection);
    let choice = keys.get(selection).unwrap();
    println!("Enjoy your {:?}!", projects[choice.as_str()]);
}

fn add_project(matches: &clap::ArgMatches<'_>) {
    let project_name = matches.value_of("project_name").unwrap();
    let project_path = matches.value_of("project_path").unwrap();
    println!("adding project {} at path {}", project_name, project_path);
}

fn open_project(matches: &clap::ArgMatches<'_>) {
    let project_name = matches.value_of("projectname").unwrap();
    let project_data = load_projects_from_data();
    
    let project = project_data.unwrap().clone();
    let hash = project.into_hash().unwrap();
    let entry = hash.get(&Yaml::from_str(&project_name));
    
    match entry {
        Some(e) => {
            let path = e.as_str().unwrap();
            println!("opening {} in VSCode at {}", project_name, path);
            Command::new("code").arg(path).output().expect("failed to open code");
        },
        None => println!("Project doesn't exist!")
    }
}
