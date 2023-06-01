use clap::{Parser, Subcommand, ArgGroup};
use sss_rs::wrapped_sharing::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;


const CREATE_ABORT: &str = "Cannot finish creating shares";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
        ArgGroup::new("dir_stem")
            .conflicts_with("outputs")
            .args(&["output_dir", "output_stem"])
        )
    )
]
#[command(group(
        ArgGroup::new("output_options")
            .conflicts_with("outputs")
            .args(&["outputs", "dir_stem"])
            .required(true)
        )
    )
]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Share {
        #[arg(help="The file to create shares from")]
        secret_input: PathBuf,

        #[arg(help="The number of shares to create")]
        shares_to_create: u8,

        #[arg(help="The number of shares needed to reconstruct the original secret")]
        shares_needed: u8,
        
        #[arg(short = 'd', long = "output-dir", help="The output directory")]
        output_dir: Option<PathBuf>,
        
        #[arg(short = 's', long = "output-stem")]
        output_stem: Option<String>,

        
        #[arg(short, long, help="List of output file names, entries must match <shares_to_create>")]
        outputs: Option<Vec<PathBuf>>,

        #[arg(short, long, help="Disable verifiable reconstruction")]
        no_confirm: bool,
    },
    Reconstruct {
        #[arg(help = "The file to write the reconstruct out to")]
        recon_output: PathBuf,

        #[arg(help = "List of files to use for reconstruction", required = true)]
        secret_inputs: Vec<PathBuf>, 

        #[arg(short, long, help="Disable verifiable reconstruction")]
        no_confirm: bool,
    },
}



fn main() {
    
    run_with_args(Cli::parse());
}

// This exists as a separate function so a GUI interface can be developed and make use of this
// in the future.
fn run_with_args(args: Cli) {
    match args.command {
        Commands::Share {
            secret_input,
            shares_to_create,
            shares_needed,
            output_dir,
            output_stem,
            outputs,
            no_confirm
        } => {
            // Create subcommand main arguments

            match std::fs::metadata(&secret_input) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        println!(
                            "'{}' is an empty file. Secret cannot be an empty file. Aborting.",
                            secret_input.as_path().to_str().unwrap()
                        );
                        println!("Aborting");
                        return;
                    }
                }
                Err(e) => {
                    println!("Error reading in secret input file '{}': {}", secret_input.as_path().to_str().unwrap(), e);
                    return;
                }
            }
            let output_paths: Vec<PathBuf> = outputs.unwrap_or_else(|| {

                let out_file_stem = output_stem.unwrap_or(
                    secret_input
                        .as_path()
                        .file_stem()
                        .expect("Invalid input file path.")
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
                (0..(shares_to_create as usize))
                    .map(|share_num| output_dir.clone().unwrap().join(format!("{}_{}.sss", out_file_stem, share_num)).to_path_buf())
                    .collect()

            });

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
            let mut dests: Vec<Box<dyn Write>> = output_paths.into_iter()
                .map(|path| {
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
            let file_src = File::open(secret_input).expect("Failed to open secret file for sharing");
            share_to_writables(file_src, &mut dests, shares_needed, shares_to_create, !no_confirm)
                .expect("Failed to share secret.");

        }
        Commands::Reconstruct {
            recon_output, 
            secret_inputs,
            no_confirm,
        } => {
            let mut src_len = 0u64;
            let mut input_files: Vec<Box<dyn Read>> = secret_inputs.into_iter()
                .map(|path| {
                    src_len = std::fs::metadata(&path).expect(&format!("Could not open file '{}'", path.to_string_lossy())).len();
                    Box::new(
                    File::open(&path).expect(&format!("Could not open file '{}'", path.to_string_lossy()))) as Box<dyn Read> 
                })
                .collect();
            
            let secret_dest = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&recon_output)
                .expect(&format!("Failed to open file {} for writing", recon_output.to_string_lossy()));
            reconstruct_from_srcs(secret_dest, &mut input_files, src_len, !no_confirm)
                .expect("Reconstruction failed");

        }
    }
}
