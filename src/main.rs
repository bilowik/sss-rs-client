use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{Arg, ArgMatches, SubCommand};
use sss_rs::wrapped_sharing::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const SUBCOMMAND_SHARE: &str = "share";
const ARG_INPUT: &str = "INPUT";
const ARG_SHARES_TO_CREATE: &str = "SHARES_TO_CREATE";
const ARG_SHARES_NEEDED: &str = "SHARES_NEEDED";
const ARG_OUTPUT_DIR: &str = "OUTPUT_DIR";
const ARG_OUTPUTS: &str = "OUTPUTS";
const ARG_OUTPUT_STEM: &str = "OUTPUT_STEM";
const ARG_NO_CONFIRM_RECON: &str = "CONFIRM_RECON";

const SUBCOMMAND_RECONSTRUCT: &str = "reconstruct";
const ARG_OUTPUT_FILE: &str = "OUTPUT_FILE";
const ARG_INPUTS: &str = "INPUTS";

// Error message constants
const CREATE_ABORT: &str = "Cannot finish creating shares, aborting";

fn main() {
    let matches = app_from_crate!()
        // Create command:
        .subcommand(
            SubCommand::with_name(SUBCOMMAND_SHARE)
                .about("Creates shares from a given file.")
                .arg(
                    Arg::with_name(ARG_INPUT)
                        .index(1)
                        .help("Sets the input file to be used to create shares.")
                        .required(true),
                )
                .arg(
                    Arg::with_name(ARG_SHARES_TO_CREATE)
                        .index(2)
                        .help("The number of shares to create.")
                        .required(true),
                )
                .arg(
                    Arg::with_name(ARG_SHARES_NEEDED)
                        .index(3)
                        .help("The number of shares needed to recreate the secret.")
                        .required(true),
                )
                .arg(
                    Arg::with_name(ARG_OUTPUT_DIR)
                        .short("d")
                        .long("out-dir")
                        .takes_value(true)
                        .default_value(".")
                        .help("The directory to output the shares and prime number")
                        .required(false),
                )
                .arg(
                    Arg::with_name(ARG_OUTPUT_STEM)
                        .short("s")
                        .long("stem")
                        .takes_value(true)
                        .help("Sets the stem of the output share files.")
                        .long_help(
                            "Sets the stem of the output share files. Example: 'file.out'
file.out.s1, file.out.s2, and so on.",
                        )
                        .required(false),
                )
                .arg(
                    Arg::with_name(ARG_OUTPUTS)
                        .short("o")
                        .long("outputs")
                        .takes_value(true)
                        .multiple(true)
                        .conflicts_with(ARG_OUTPUT_STEM)
                        .help("List of files to output shares to."),
                )
                .arg(
                    Arg::with_name(ARG_NO_CONFIRM_RECON)
                        .short("n")
                        .long("no-confirm")
                        .takes_value(false)
                        .help(
                            "Skip adding a hash to the end of the input to confirm reconstruction",
                        )
                        .required(false),
                ),
        )
        // Reconstruct command:
        .subcommand(
            SubCommand::with_name(SUBCOMMAND_RECONSTRUCT)
                .about("Reconstructs shares into a single file secret.")
                .arg(
                    Arg::with_name(ARG_NO_CONFIRM_RECON)
                        .short("n")
                        .long("no-confirm")
                        .takes_value(false)
                        .help("Don't assume there's a hash at the end of the reconstructed file.")
                        .required(false),
                )
                .arg(
                    Arg::with_name(ARG_OUTPUT_FILE)
                        .index(1)
                        .help("The file to output the generated secret.")
                        .required(true),
                )
                .arg(
                    Arg::with_name(ARG_INPUTS)
                        .index(2)
                        .multiple(true)
                        .help("List of files to reconstruct from.")
                        .required(true),
                ),
        )
        .get_matches();

    run_with_args(&matches);
}

// This exists as a separate function so a GUI interface can be developed and make use of this
// in the future.
fn run_with_args(args: &ArgMatches) {
    match args.subcommand() {
        (SUBCOMMAND_SHARE, Some(sub_matches)) => {
            // Create subcommand main arguments
            let file = sub_matches.value_of(ARG_INPUT).unwrap();

            match std::fs::metadata(file) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        println!(
                            "'{}' is an empty file. Secret cannot be an empty file. Aborting.",
                            file
                        );
                        println!("Aborting");
                        return;
                    }
                }
                Err(e) => {
                    println!("Error reading in secret input file '{}': {}", file, e);
                    println!("Aborting");
                    return;
                }
            }

            let out_file_stem = sub_matches.value_of(ARG_OUTPUT_STEM).unwrap_or(
                Path::new(sub_matches.value_of(ARG_INPUT).unwrap())
                    .file_stem()
                    .expect("Invalid input file path.")
                    .to_str()
                    .unwrap(),
            );

            let out_dir = sub_matches.value_of(ARG_OUTPUT_DIR).unwrap_or(".");

            let shares_to_create = sub_matches
                .value_of(ARG_SHARES_TO_CREATE)
                .unwrap()
                .parse::<u8>()
                .unwrap();

            let shares_needed = sub_matches
                .value_of(ARG_SHARES_NEEDED)
                .unwrap_or(sub_matches.value_of(ARG_SHARES_TO_CREATE).unwrap())
                .parse::<u8>()
                .unwrap();

            let confirm = sub_matches.is_present(ARG_NO_CONFIRM_RECON);

            // Error checking of number of shares values
            if shares_to_create < shares_needed {
                println!(
                    "Shares to create must be greater than or equal to shares needed:
                         Shares to create: {}
                         Shares needed for reconstruction: {}",
                    shares_to_create, shares_needed
                );
                println!("{}", CREATE_ABORT);
                return;
            }
            if shares_to_create < 2 || shares_needed < 2 {
                println!(
                    "At least two shares are needed to split a secret:
                         Shares to create: {}
                         Shares needed for reconstruction: {}",
                    shares_to_create, shares_needed
                );
                println!("{}", CREATE_ABORT);
                return;
            }


            // Create a vec of writable files for sharing.
            let mut dests: Vec<Box<dyn Write>> = (0..(shares_to_create as usize))
                .map(|share_num| {
                    let path =
                        Path::new(out_dir).join(format!("{}_{}.sss", out_file_stem, share_num));
                    Box::new(
                        std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(&path)
                            .expect(&format!(
                                "Failed to open file '{}' for writing.",
                                path.as_path().to_str().unwrap()
                            )),
                    ) as Box<dyn Write>
                })
                .collect();
            let file_src = File::open(file).expect("Failed to open secret file for sharing");
            share_to_writables(file_src, &mut dests, shares_needed, shares_to_create, confirm)
                .expect("Failed to share secret.");

        }
        (SUBCOMMAND_RECONSTRUCT, Some(sub_matches)) => {
            let mut src_len = 0u64;
            let mut input_files: Vec<Box<dyn Read>> = sub_matches
                .values_of(ARG_INPUTS)
                .expect("No input values received?")
                .map(|path| {
                    src_len = std::fs::metadata(&path).expect(&format!("Could not open file '{}'", &path)).len();
                    Box::new(
                    File::open(&path).expect(&format!("Could not open file '{}'", &path))) as Box<dyn Read> 
                })
                .collect();
            
            let confirm = sub_matches.is_present(ARG_NO_CONFIRM_RECON);

            let secret_out = sub_matches.value_of(ARG_OUTPUT_FILE).unwrap();
            let secret_dest = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(secret_out)
                .expect(&format!("Failed to open file {} for writing", secret_out));
            reconstruct_from_srcs(secret_dest, &mut input_files, src_len, confirm)
                .expect("Reconstruction failed");

        }
        _ => (), // No subcommand was run?
    }
}
