extern crate clap;
extern crate dialoguer;
extern crate yaml_rust;
extern crate dirs;
extern crate colored;

use clap::{Arg, App, SubCommand};
use dialoguer::{theme::ColorfulTheme, Select, Input};
use yaml_rust::{YamlLoader, Yaml, Yaml::Hash, YamlEmitter};
use std::fs::{File, OpenOptions};
use std::io::{Read, Error, ErrorKind, Write};
use std::vec::Vec;
use std::result::Result;
use std::process::Command;
use std::path::PathBuf;
use colored::*;

fn main() {
    let matches = App::new("Projects")
        .version("0.1")
        .author("Aahlad Madireddy")
        .about("easy project management")
        .subcommand(SubCommand::with_name("open")
                .about("open a project")
                .arg(Arg::with_name("projectname")
                        .required(false)
                        .index(1)))
        .subcommand(SubCommand::with_name("remove")
                .about("remove a bookmark")
                .arg(Arg::with_name("projectname")
                        .required(false)
                        .index(1)))
        .subcommand(SubCommand::with_name("rm")
                .about("remove a bookmark")
                .arg(Arg::with_name("projectname")
                        .required(false)
                        .index(1)))
        .subcommand(SubCommand::with_name("list")
                .about("list project tracked with project manager"))
        .subcommand(SubCommand::with_name("ls")
                .about("list project tracked with project manager"))
        .subcommand(SubCommand::with_name("add")
                .about("add project to track")
                .arg(Arg::with_name("project_name")
                        .required(false)
                        .index(1))
                .arg(Arg::with_name("project_path")
                        .required(false)
                        .index(2)))
        .get_matches();

    match matches.subcommand() {
        ("open", Some(sub)) => {
            open_project(Option::from(sub))
        } ,
        ("remove", Some(sub)) | ("rm", Some(sub)) => {
            remove_project(&sub)
        },
        ("list", Some(_)) | ("ls", Some(_)) => {
            list_projects()
        },
        ("add", Some(sub)) => {
            add_project(&sub)
        },
        (&_, None) => {
            open_project(None)
        },
        _ => println!("{}", matches.usage()),
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

    return match f {
        Ok(mut contents) => {
            let mut s = String::new();
            contents.read_to_string(&mut s)?;
            let docs = YamlLoader::load_from_str(&s).unwrap();
            if docs.len() == 0 {
                return Err(Error::new(ErrorKind::Other, "No yaml in file, may be corrupted"))
            }
            Ok(docs[0].to_owned())
        },
        Err(e) => {
            if let Some(raw_err) = e.raw_os_error() {
                match raw_err {
                    2 => {
                        println!("file doesn't exist, creating");
                        std::fs::File::create(config_path)?;
                        let v: Yaml = Yaml::from_str("");
                        Ok(v)
                    },
                    _ => {
                        println!("Other error reading data file");
                        Err(e)
                    }
                }
            } else {
                Err(e)
            }
        }
    }
}

fn get_keys_from_project_data(projects: &Yaml) -> Vec<String> {
    if let Yaml::Hash(hash) = projects {
        let mut keys: Vec<String> = Vec::new();
        
        for (k, _v) in hash {
            let key = k.as_str().unwrap().to_owned();
            keys.push(key);
        }
    
        return keys
    }
    
    return Vec::new()
}

fn display_selection(keys: &Vec<String>) -> usize {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select project")
        .default(0)
        .items(keys.as_slice())
        .interact()
        .unwrap();

    selection
}

fn list_projects() {
    let projects_yaml = load_projects_from_data();
    let projects: Yaml; 

    match projects_yaml {
        Ok(pr) => {
            projects = pr.clone();
        },
        Err(r) => {
            println!("error: {}", r);
            return
        },
    }

    let keys = get_keys_from_project_data(&projects);

    if let Hash(hash) = projects {
        for i in keys {   
            let location = hash.get(&Yaml::from_str(&i)).unwrap().as_str().unwrap();
            println!("{} - {}", i.as_str().green(), location);
        }
    }
}

fn write_to_data_file(yaml: &Yaml) -> bool {
    let mut home_directory = dirs::home_dir().unwrap();
    let mut f: File;
    home_directory.push(".projectman");

    f = OpenOptions::new().write(true).open(home_directory).unwrap();
    f.set_len(0).unwrap();
    let mut raw = String::new();
    let mut emitter = YamlEmitter::new(&mut raw);
    emitter.dump(&yaml).unwrap();
    let res = f.write(raw.as_bytes());
    return match res {
        Ok(_) => true,
        Err(_) => false
    }
}

fn add_project(matches: &clap::ArgMatches<'_>) {
    let project_name_match = matches.value_of("project_name");
    let project_path_match = matches.value_of("project_path");
    let project_name: String;
    let project_path: &str;

    match project_name_match {
        Some(p) => project_name = String::from(p),
        None => project_name = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(&"Enter name of project")
                .interact()
                .unwrap()
    }

    match project_path_match {
        Some(p) => project_path = p,
        None => project_path = "."
    }

    let canon_path_buf = std::fs::canonicalize(PathBuf::from(project_path)).unwrap();
    let canon_path = canon_path_buf.to_str().unwrap();

    let data = load_projects_from_data();
    match data {
        Ok(mut data) => {
            if let Hash(hash) = &mut data {
                let project_name_yaml = Yaml::from_str(project_name.as_str());
                let project_path_yaml = Yaml::from_str(canon_path);
                let mut selection: usize = 0;
                let mut exists: bool = false;

                if hash.contains_key(&project_name_yaml) {
                    exists = true;
                    let select_items = ["No", "Yes"];
                    selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("A project with that name already exists. Overwrite?")
                        .items(&select_items)
                        .default(0)
                        .interact()
                        .unwrap();
                }

                if selection == 1 || !exists {
                    hash.insert(project_name_yaml, project_path_yaml);
                    let written = write_to_data_file(&data);
                    if written {
                        println!("{} Added project '{}' at path {}", "\u{2714}".green(), project_name, canon_path);
                    } else {
                        println!("{}", "Couldn't write to config file".red());
                    }
                } else {
                    println!("{} Skipping overwrite. No changes made.", "\u{274C}".red())
                }
            }
        },
        Err(e) => {
            println!("Error loading current projects {}", e)
        }
    }
}

fn run_open_command(p: &str, path: &str) {
    println!("Opening project {} at {}", p, path);
    Command::new("code").arg(path).output().expect("failed to open code");
}

fn open_with_selector(project: &Yaml) {
    if let Yaml::Hash(hash) = project {
        let keys = get_keys_from_project_data(&project);
        let selection = display_selection(&keys);
        let choice = keys.get(selection).unwrap();
        let chosen_project_name = choice.as_str();
        let chosen_path = hash.get(&Yaml::from_str(&chosen_project_name));
        match chosen_path {
            Some(e) => {
                let path = e.as_str().unwrap();
                run_open_command(chosen_project_name, path);
            },
            None => println!("No configured path for that project"),
        }
    }
}

fn open_project(arg_matches: Option<&clap::ArgMatches<'_>>) {
    let project_data = load_projects_from_data();
    let project = project_data.unwrap();

    if arg_matches.is_none() {
        open_with_selector(&project);
    }

    if let Some(matches) = arg_matches {
        let project_name = matches.value_of("projectname");

        if let Yaml::Hash(hash) = &project {
            match project_name {
                Some(p) => {
                    let entry = hash.get(&Yaml::from_str(&p));
                    match entry {
                        Some(e) => {
                            let path = e.as_str().unwrap();
                            run_open_command(p, path);
                        },
                        None => println!("Project doesn't exist!")
                    }
                },
                None => open_with_selector(&project)
            }
        }
    }
}

fn remove_with_selector(project: &mut Yaml) {
    let keys = get_keys_from_project_data(&project);
    if let Yaml::Hash(hash) = project {
        let selection = display_selection(&keys);
        let choice = keys.get(selection).unwrap();
        let chosen_project_name = choice.as_str();
        let removed_path = hash.remove(&Yaml::from_str(&chosen_project_name));
        println!("Removed bookmark {} at {}", chosen_project_name, removed_path.unwrap().as_str().unwrap());
    }
}

fn remove_project(arg_matches: &clap::ArgMatches<'_>) {
    let project_data = load_projects_from_data();
    let mut project = project_data.unwrap();

    let project_to_rm = arg_matches.value_of("projectname");
    match project_to_rm {
        Some(t) => {
            if let Yaml::Hash(hash) = &mut project {
                let removed = hash.remove(&Yaml::from_str(t));
                println!("Removed bookmark {} at {}", t, removed.unwrap().as_str().unwrap());
            }
        },
        None => {
            remove_with_selector(&mut project);
        }
    }

    write_to_data_file(&project);
}
