use clap::{Parser, Subcommand};
use sss_rs::wrapped_sharing::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const CREATE_ABORT: &str = "Cannot finish creating shares";

const BUF_SIZE: usize = 8192;

//struct BetterBufReader<T: Read> {
//    size: usize,
//    inner: BufReader<T>,
//}
//
//impl<T: Read> BetterBufReader<T> {
//    pub fn new(inner: BufReader<T>, size: usize) -> Self {
//        Self {
//            inner,
//            size,
//        }
//    }
//
//    pub fn next(&mut self) -> Result<&[u8], std::io::Error> {
//        self.inner.fill_buf()
//    }
//}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Share {
        #[arg(help = "The file to create shares from")]
        secret_input: PathBuf,

        #[arg(help = "The number of shares to create")]
        shares_to_create: u8,

        #[arg(help = "The number of shares needed to reconstruct the original secret")]
        shares_needed: u8,

        #[arg(short = 'd', long = "output-dir", help = "The output directory")]
        output_dir: PathBuf,

        #[arg(
            short = 's',
            long = "output-stem",
            help = "The filename stem for the shares (<OUTPUT_STEM>_<i>.<output_ext>), defaults to the stem of the input file."
        )]
        output_stem: Option<String>,

        #[arg(
            short = 'e',
            long = "output-ext",
            default_value = "sss",
            help = "The filename ext for the shares (<output_stem>_<i>.<OUTPUT_EXT>), defaults to 'sss'"
        )]
        output_ext: String,

        #[arg(short, long, help = "Disable verifiable reconstruction")]
        no_confirm: bool,
    },
    Reconstruct {
        #[arg(help = "The file to write the reconstruct out to")]
        recon_output: PathBuf,

        #[arg(
            help = "List of files to use for reconstruction",
            required = true,
            value_delimiter = ' '
        )]
        secret_inputs: Vec<PathBuf>,

        #[arg(short, long, help = "Disable verifiable reconstruction")]
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
            output_ext,
            no_confirm,
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
                    println!(
                        "Error reading in secret input file '{}': {}",
                        secret_input.as_path().to_str().unwrap(),
                        e
                    );
                    return;
                }
            }
            let output_paths: Vec<PathBuf> = {
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
                    .map(|share_num| {
                        output_dir
                            .join(format!("{}_{}.{}", out_file_stem, share_num, output_ext))
                            .to_path_buf()
                    })
                    .collect()
            };

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
            let mut sharer = output_paths
                .into_iter()
                .map(|path| {
                    std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&path)
                        .expect(&format!(
                            "Failed to open file '{}' for writing.",
                            path.as_path().to_str().unwrap()
                        ))
                })
                .fold(Sharer::builder(), |builder, f| builder.with_output(f))
                .with_verify(!no_confirm)
                .with_shares_required(shares_needed)
                .build()
                .expect("Failed to start sharing");

            let mut file_src = BufReader::with_capacity(
                BUF_SIZE,
                File::open(secret_input).expect("Failed to open secret file for sharing"),
            );
            // do-while loop that will loop through BUF_SIZE bytes of the file until it is empty.
            while {
                let len = {
                    let curr_buf = file_src.fill_buf().expect("IO Error reading from secret");
                    sharer
                        .update(curr_buf)
                        .expect("IO Error writing to output files");
                    curr_buf.len()
                };
                file_src.consume(BUF_SIZE);
                len > 0
            } {}
            sharer.finalize().expect("Failed to finalize the sharing");
        }
        Commands::Reconstruct {
            recon_output,
            secret_inputs,
            no_confirm,
        } => {
            let lens = secret_inputs
                .iter()
                .map(|p| {
                    p.metadata()
                        .expect(&format!(
                            "IO Error getting length of file of input src: {}",
                            p.to_string_lossy()
                        ))
                        .len()
                })
                .collect::<Vec<u64>>();
            let mut curr_len = lens[0];
            for (path, len) in secret_inputs.iter().zip(lens.into_iter()) {
                if curr_len != len {
                    println!("Input files must have identical lengths, file '{}' with len '{}' differed in length of previous files len '{}'",
                             path.to_string_lossy(), len, curr_len);
                    return;
                }

                curr_len = len;
            }

            let mut input_readers: Vec<BufReader<File>> = secret_inputs
                .iter()
                .map(|path| {
                    BufReader::with_capacity(
                        BUF_SIZE,
                        File::open(&path)
                            .expect(&format!("Could not open file '{}'", path.to_string_lossy())),
                    )
                })
                .collect();

            let secret_dest = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&recon_output)
                .expect(&format!(
                    "Failed to open file {} for writing",
                    recon_output.to_string_lossy()
                ));

            let mut reconstructor = Reconstructor::new(secret_dest, !no_confirm);

            while {
                let chunks: Vec<&[u8]> = input_readers
                    .iter_mut()
                    .map(|buf_reader| {
                        buf_reader
                            .fill_buf()
                            .expect("IO error occurred while reading a share")
                    })
                    .collect();

                let num_bytes = reconstructor
                    .update(chunks)
                    .expect("An error occured during reconstruction");
                input_readers
                    .iter_mut()
                    .for_each(|buf_reader| buf_reader.consume(BUF_SIZE));
                num_bytes > 0
            } {}
        }
    }
}
