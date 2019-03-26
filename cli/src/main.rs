mod error;

use ferrous_chloride::{parse_reader, MergeBehaviour};
use std::fs::File;
use std::io::{self, Read, Write};

use crate::error::Error;

use clap::{
    crate_authors, crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};

fn main() -> Result<(), Error> {
    let args = make_parser().get_matches();
    run_subcommand(&args)?;
    Ok(())
}

fn run_subcommand(args: &ArgMatches) -> Result<(), Error> {
    match args.subcommand() {
        ("parse", Some(args)) => run_parse(args),
        (unknown, _) => Err(Error::UnknownCommand(unknown.to_string()))?,
    }
}

fn run_parse(args: &ArgMatches) -> Result<(), Error> {
    let input = args
        .value_of("input")
        .expect("Required argument is provided");
    let output = args
        .value_of("output")
        .expect("Required argument is provided");

    let no_merge = args.is_present("no_merge");

    let parsed = {
        let input = input_reader(input)?;

        let merge_behaviour = if no_merge {
            None
        } else {
            Some(MergeBehaviour::Error)
        };
        parse_reader(input, merge_behaviour)?
    };

    // Write
    {
        let mut output = output_writer(output)?;
        output.write_all(format!("{:#?}", parsed).as_bytes())?;
    }

    Ok(())
}

/// Gets a `Read` depending on the path. If the path is `-`, read from STDIN
fn input_reader(path: &str) -> Result<Box<Read>, Error> {
    match path {
        "-" => Ok(Box::new(io::stdin())),
        path => {
            let file = File::open(path)?;
            Ok(Box::new(file))
        }
    }
}

/// Gets a `Write` depending on the path. If the path is `-`, write to STDOUT
fn output_writer(path: &str) -> Result<Box<Write>, Error> {
    match path {
        "-" => Ok(Box::new(io::stdout())),
        path => {
            let file = File::create(path)?;
            Ok(Box::new(file))
        }
    }
}

fn make_parser<'a, 'b>() -> App<'a, 'b>
where
    'a: 'b,
{
    let parse = SubCommand::with_name("parse")
        .about("Parse a HCL file and print out the abstract syntax tree")
        .arg(
            Arg::with_name("no_merge")
                .long("no-merge")
                .help("Do not merge value after parsing")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("input")
                .index(1)
                .help(
                    "Specifies the path to read the HCL from. \
                     Use - to refer to STDIN",
                )
                .takes_value(true)
                .value_name("input_path")
                .empty_values(false)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("output")
                .index(2)
                .help(
                    "Specifies the path to write the parsed AST to. \
                     Use - to refer to STDOUT",
                )
                .takes_value(true)
                .value_name("output_path")
                .empty_values(false)
                .default_value("-"),
        );

    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::InferSubcommands)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::NextLineHelp)
        .about("HCL Parser")
        .subcommand(parse)
}
