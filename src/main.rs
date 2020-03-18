extern crate clap;
use clap::{Arg, App, SubCommand};

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

    if let Some(matches) = matches.subcommand_matches("open") {
        open_project(matches);
    } else if let Some(_matches) = matches.subcommand_matches("list") {
        list_projects();
    } else if let Some(matches) = matches.subcommand_matches("add") {
        add_project(matches);
    }
}

fn list_projects() {
    println!("listing projects");
}

fn add_project(matches: &clap::ArgMatches<'_>) {
    let project_name = matches.value_of("project_name").unwrap();
    let project_path = matches.value_of("project_path").unwrap();
    println!("adding project {} at path {}", project_name, project_path);
}

fn open_project(matches: &clap::ArgMatches<'_>) {
    let project_name = matches.value_of("projectname").unwrap();
    println!("opening project {}", project_name);
}
