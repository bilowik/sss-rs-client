/* TODO list: 
 *  - Clean up error handling and feedback. Should not be split between the arg handler and the
 *      create/reconstruct functions.
 *      - As an aside to this, changing the 'stem' argument to a list of strings and then
 *      generating them and confirming they exist and are readable (or are writable), is probably
 *      the easiest path to this goal.
 *   
 *
 *
 *
 *
 *
 *
 *
 *
 *
 */



use sss_rs::*;
use sss_rs::geometry::Point;
use num_bigint_dig::{BigInt, RandPrime};
use rand::rngs::StdRng;
use rand::FromEntropy;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::error::Error;
use clap::{Arg, SubCommand, ArgMatches};
use clap::{app_from_crate, crate_name, crate_authors, crate_version, crate_description};
const SUBCOMMAND_CREATE: &str = "create";
const ARG_INPUT: &str = "INPUT";
const ARG_SHARES_TO_CREATE: &str = "SHARES_TO_CREATE";
const ARG_SHARES_NEEDED: &str = "SHARES_NEEDED";
const ARG_OUTPUT_STEM: &str = "OUTPUT_STEM";
const ARG_WITH_PASSWORD: &str = "WITH_PASSWORD";
const ARG_OUTPUT_PRIME: &str = "OUTPUT_PRIME";
const ARG_OUTPUT_DIR: &str = "OUTPUT_DIR";
const ARG_CONFIRM_RECON: &str = "CONFIRM_RECON";

const SUBCOMMAND_RECONSTRUCT: &str = "reconstruct";
const ARG_OUTPUT_FILE: &str = "OUTPUT_FILE";
const ARG_PRIME_INPUT_FILE: &str = "PRIME_INPUT_FILE";
const ARG_INPUT_STEM: &str = "INPUT_STEM";
const ARG_INPUT_DIR: &str = "INPUT_DIR";


// Error message constants
const CREATE_ABORT: &str = "Cannot finish creating shares, aborting";
const RECONSTRUCT_ABORT: &str = "Cannot finish reconstruction, aborting";

fn main() {

    let matches = app_from_crate!()
                            // Create command:
                            .subcommand(SubCommand::with_name(SUBCOMMAND_CREATE)
                                .about("Creates shares from a given file.")
                                .arg(Arg::with_name(ARG_INPUT)
                                     .index(1)
                                     .help("Sets the input file to be used to create shares.")
                                     .required(true))
                                .arg(Arg::with_name(ARG_SHARES_TO_CREATE)
                                     .index(2)
                                     .help("The number of shares to create.")
                                     .required(true))
                                .arg(Arg::with_name(ARG_SHARES_NEEDED)
                                     .index(3)
                                     .help("The number of shares needed to recreate the secret.")
                                     .required(false))
                                .arg(Arg::with_name(ARG_OUTPUT_DIR)
                                     .short("d")
                                     .long("out-dir")
                                     .takes_value(true)
                                     .help("The directory to output the shares and prime number")
                                     .required(false))
                                .arg(Arg::with_name(ARG_OUTPUT_STEM)
                                     .short("s")
                                     .long("stem")
                                     .takes_value(true)
                                     .help("Sets the stem of the output share files.")
                                     .long_help(
"Sets the stem of the output share files. Example: 'file.out'
file.out.s1, file.out.s2, and so on.")
                                     .required(false))
                                .arg(Arg::with_name(ARG_WITH_PASSWORD)
                                     .short("p")
                                     .long("password")
                                     .help("Prompts for a password input to shuffle the shares.")
                                     .required(false))
                                .arg(Arg::with_name(ARG_OUTPUT_PRIME)
                                     .long("prime-file")
                                     .takes_value(true)
                                     .help(
                                 "Sets the output file for the generated prime.")
                                     .required(false))
                                .arg(Arg::with_name(ARG_CONFIRM_RECON)
                                     .short("c")
                                     .long("confirm")
                                     .takes_value(false)
                                     .help("Confirm the shares properly reconstruct to the secret")
                                     .required(false)))


                            // Reconstruct command:
                            .subcommand(SubCommand::with_name(SUBCOMMAND_RECONSTRUCT)
                                    .about("Reconstructs shares into a single file secret")
                                    .arg(Arg::with_name(ARG_INPUT_STEM)
                                         .index(1)
                                         // TODO: Allow for multiple files, or assume stem when
                                         // it's just one.
                                         .help("The stem of the share files")
                                         .required(true))
                                    .arg(Arg::with_name(ARG_SHARES_NEEDED)
                                         .index(2)
                                         .help("The number of shares needed to recreate the secret")
                                         .required(true))
                                    .arg(Arg::with_name(ARG_OUTPUT_FILE)
                                         .index(3)
                                         .help("The file to output the generated secret")
                                         .required(false))
                                    .arg(Arg::with_name(ARG_INPUT_DIR)
                                         .short("d")
                                         .long("input-dir")
                                         .takes_value(true)
                                         .help("The directory the shares are located in")
                                         .required(false))
                                    .arg(Arg::with_name(ARG_PRIME_INPUT_FILE)
                                         .long("prime-file")
                                         .takes_value(true)
                                         .help(
                                    "The file that contains the outputted prime from share creation")
                                         .required(false))
                                    .arg(Arg::with_name(ARG_WITH_PASSWORD)
                                         .short("p")
                                         .long("password")
                                         .help("Prompts for a password to unshuffle the shares")
                                         .required(false)))
                            .get_matches();

    run_with_args(&matches);


}


// This exists as a separate function so a GUI interface can be developed and make use of this 
// in the future.
fn run_with_args(args: &ArgMatches) {
    match args.subcommand() {
        (SUBCOMMAND_CREATE, Some(sub_matches)) => {
            


            // SOme defaults are declared here due to reference issues
            let default_prime_out_file = format!("{}.prime",
                                                sub_matches.value_of(ARG_INPUT).unwrap());
            let default_out_dir = ".";

            //Create subcommand main arguments
            let file = sub_matches.value_of(ARG_INPUT).unwrap();
   
            
            match std::fs::metadata(file) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        println!("'{}' is an empty file. Secret cannot be an empty file. Aborting.", 
                                 file);
                        println!("Aborting");
                        return;
                    }
                },
                Err(e) => {
                    println!("Error reading in secret input file '{}': {}", file, e);
                    println!("Aborting");
                    return;
                }
            }
            


            let out_file_stem = sub_matches.value_of(ARG_OUTPUT_STEM)
                                    .unwrap_or(sub_matches.value_of(ARG_INPUT).unwrap());

            let out_dir = sub_matches.value_of(ARG_OUTPUT_DIR).unwrap_or(default_out_dir);
            if let ValidPathType::ExistingFile | ValidPathType::Invalid | ValidPathType::NonExisting = 
                check_path(out_dir) {

                // Not given a valid output dir, print error and exit
                println!("'{}' is not a valid directory", out_dir);
                println!("{}", CREATE_ABORT);
                return;
            }

            let shares_to_create = sub_matches.value_of(ARG_SHARES_TO_CREATE)
                                                        .unwrap().parse::<usize>().unwrap();

            let shares_needed = sub_matches.value_of(ARG_SHARES_NEEDED).unwrap_or(
                        sub_matches.value_of(ARG_SHARES_TO_CREATE).unwrap()).parse::<usize>().unwrap();
  

            // Error checking of number of shares values 
            if shares_to_create < shares_needed {
                println!("Shares to create must be greater than or equal to shares needed:
                         Shares to create: {}
                         Shares needed for reconstruction: {}",
                         shares_to_create,
                         shares_needed);
                println!("{}", CREATE_ABORT);
                return;
            }
            if shares_to_create < 2 || shares_needed < 2 {
                println!("At least two shares are needed to split a secret:
                         Shares to create: {}
                         Shares needed for reconstruction: {}",
                         shares_to_create,
                         shares_needed);
                println!("{}", CREATE_ABORT);
                return;
            }





            let pass = sub_matches.is_present(ARG_WITH_PASSWORD);

            
            let prime_out_file = sub_matches.value_of(ARG_OUTPUT_PRIME)
                                                        .unwrap_or(&default_prime_out_file);
            let prime_bits = 64;

            match create_from_file(file,
                             out_file_stem,
                             out_dir,
                             shares_to_create,
                             shares_needed,
                             pass,
                             prime_out_file,
                             prime_bits) {
                Ok(_) => {
                    println!("Shares created and output to directory '{}'", out_dir);
                }
                Err(_) => {
                    // Error handling will likely be moved here in the future, for now the function
                    // handles the error and prints out the information
                    // TODO: Find a way to wrap around the errors generated within the
                    // share/reconstruct functions with addtional infromation from the context
                },
            }

            // If the user specified the confirmation flag, do a test reconstruction and compare
            // the original and the reconstructed secret for equivalence. 
            if sub_matches.is_present(ARG_CONFIRM_RECON) {
                let confirm_file_out = ".confirm_recon";
                // Dry run the reconstruction process and confirm that the outputted secret is
                // exactly the same as the input secret
                println!("Beginning reconstruction test run, if you supplied a password, \
                         you will have to reinput it. Incorrect passwords will result in a \
                         reconstruction failure.");
                if let Err(e) = reconstruct_from_shares(out_file_stem,
                                        out_dir,
                                        prime_out_file,
                                        pass,
                                        shares_needed,
                                        confirm_file_out) {
                    println!("Reconstruction process failed: {}", e);
                    println!("Could not complete reconstruction confirmation, aborting.");
                    return;
                }
    
                let orig_bytes = match std::fs::read(file) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        println!("Error reading in original secret '{}': {}", file, e);
                        println!("Could not complete reconstruction confirmation, aborting.");
                        return;
                    }
                };

                let recon_bytes = match std::fs::read(confirm_file_out) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        println!("Error reading in reconstructed secret '{}': {}", confirm_file_out, e);
                        println!("Could not complete reconstruction confirmation, aborting.");
                        return;
                    }
                };
                
                if orig_bytes != recon_bytes {
                    println!("Test reconstruction FAILED! Did you input the right password?");
                }
                else {
                    println!("Test reconstruction PASSED! The generates shares were successfully \
                    reconstructed into the original secret.");
                }

                if let Err(e) = std::fs::remove_file(confirm_file_out) {
                    println!("Could not clean up confirmation test file '{}': {}", confirm_file_out, e);
                }

            }
                
        },
        (SUBCOMMAND_RECONSTRUCT, Some(sub_matches)) => {
            


            // Main args
            let stem = sub_matches.value_of(ARG_INPUT_STEM).unwrap();

            let default_input_dir = ".";
            let input_dir = sub_matches.value_of(ARG_INPUT_DIR).unwrap_or(default_input_dir);

            let default_prime_in = format!("{}.prime", stem);
            let prime_in = sub_matches.value_of(ARG_PRIME_INPUT_FILE)
                    .unwrap_or(default_prime_in.as_ref());
            let pass = sub_matches.is_present(ARG_WITH_PASSWORD);
            let shares_needed = match sub_matches.value_of(ARG_SHARES_NEEDED).unwrap().parse::<usize>() {
                Ok(val) => val,
                Err(e) => {
                    println!("Invalid number of shares, must be an positive number: {}", e);
                    println!("{}", CREATE_ABORT);
                    return;
                }
            };

            let default_secret_out = format!("{}.out", stem);

            // If an output file isn't specified, '.out' is appended to the STEM argument,
            // If an output file is specified, it is verified to be a valid path, and 
            // confirmation is requested when overwriting an existing file
            let secret_out = match sub_matches.value_of(ARG_OUTPUT_FILE) {
                Some(path) => path,
                None => default_secret_out.as_ref(),
            };

            // Check the path and confirm overwrite if the file already exists, or exit if it's not
            // a valid file path
            match check_path(secret_out) {

                ValidPathType::ExistingFile => {
                    // File already exists, confirm overwerite
                    let confirmation: bool;
                    match dialoguer::Confirmation::new() 
                        .with_text(format!("'{}' already exists, overwrite?", secret_out).as_ref())
                        .interact() {

                        Ok(answer) => {
                            confirmation = answer;
                        },
                        Err(e) => {
                            println!("Error creating overwrite dialogue: {}", e);
                            println!("{}", CREATE_ABORT);
                            return;
                        }
                    }
       
                    if !confirmation {
                        // The user did not confirm
                        println!("Overwrite of file '{}' not confirmed", secret_out);
                        println!("{}", CREATE_ABORT);
                        return;
                    }
                },

                ValidPathType::ExistingDir => {
                    // The output file can't be a directory, return
                    println!("'{}' is a directory, not a file", secret_out);
                    println!("{}", CREATE_ABORT);
                    return;
                },

                ValidPathType::NonExisting => {
                    // The path is valid and the file doesn't already exist
                },

                ValidPathType::Invalid => {
                    // The path is not valid
                    println!("'{}' is not a valid path", secret_out);
                    println!("{}", CREATE_ABORT);
                    return;
                }

            }

            
        
            match reconstruct_from_shares(stem,
                                    input_dir,
                                    prime_in,
                                    pass,
                                    shares_needed,
                                    secret_out) {
                Ok(_) => {
                    println!("Secret reconstructed at {}", secret_out);
                },
                Err(_) => {
                    // Error handling will likely be moved here in the future, for now the function
                    // handles the error and prints out the information
                }
            }
        },
        _ => () // No subcommand was run
    } 



}



fn create_from_file(file: &str, 
                    out_file_stem: &str, 
                    out_dir: &str,
                    shares_to_create: usize, 
                    shares_needed: usize, 
                    pass: bool, 
                    prime_out_file: &str, 
                    prime_bits: usize) -> Result<(), Box<dyn Error>> {

    let mut rand = StdRng::from_entropy();
    let prime: BigInt = rand.gen_prime(prime_bits).into();
    let co_max_bits = 16usize;

    let mut in_file = File::open(file).unwrap(); // Should not panic, since the file was checked already

    let mut secret = Vec::<u8>::new();
    in_file.read_to_end(&mut secret)?;

    let mut share_lists = create_share_lists_from_secrets(secret.as_slice(), &prime, shares_needed, 
                                                      shares_to_create, co_max_bits)?;
    if pass { 
        let password = dialoguer::PasswordInput::new()
                                        .with_prompt("Password for shares")
                                        .with_confirmation("Confirm password", "Passwords don't match")
                                        .interact()?;
        share_lists = shuffle_share_lists(share_lists, password.as_ref(), ShuffleOp::Shuffle);
    }


    // Generate the file output paths
    let file_out_paths = generate_share_file_paths(out_dir, out_file_stem, shares_to_create);
    

    // The format of the shares will be as so:
    //      First 8 bytes: The number of individual shares in the file
    //      Then for each share:
    //          4 bytes: The number of bytes for that given share
    //          N bytes: Determined by the previous 4 bytes, the share itself.
    let mut i = 0;
    for share_list in share_lists {

        // Create the file for this share
        // If the file cannot be created, print error information and abort.
        let mut file_out = match File::create(&file_out_paths[i]) {
            Ok(file) => file,
            Err(e) => {
                println!("Error creating share file '{}': {}", &file_out_paths[i], e);
                println!("{}", CREATE_ABORT);
                return Err(Box::new(e));
            }
        };
       
        // Write out the number of shares into the first 8 bytes of the file. Must convert to u64
        // to ensure cross platform functionality.
        file_out.write_all(&(share_list.len() as u64).to_be_bytes())?;

        i = i + 1;
        let mut j = 0;
        for share in share_list {
            if !share.y().is_whole() {
                panic!("Remove me");
            }
            let bytes_from_share = share.y().get_numerator().to_signed_bytes_be();

            // The size must be written as a u32 to be consistent across platforms. For len to be
            // above the size of u32 is an extremely unlikely scenario (I hope).
            file_out.write_all(&(bytes_from_share.len() as u32).to_be_bytes())?;
            file_out.write_all(bytes_from_share.as_slice())?;

            j = j + 1;
        }
    }

    // Now write the prime out to a file
    // If the prime file cannot be written, print error information and abort reconstruction
    let prime_out_path = generate_prime_file_path(out_dir, prime_out_file);
    let mut prime_out = match File::create(&prime_out_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Error creating prime input file '{}': {}", prime_out_path, e);
            println!("{}", CREATE_ABORT);
            return Err(Box::new(e));
        }
    };
    let prime_bytes = prime.to_signed_bytes_be();
    prime_out.write_all(&(prime_bytes.len() as u32).to_be_bytes())?;
    prime_out.write_all(prime_bytes.as_slice())?;


    Ok(())
}

// TODO: When multiple files are allowed, use a tuple with two options, one with the single stem and
// the other with a list of strs 
fn reconstruct_from_shares(stem: &str, 
                           input_dir: &str,
                           prime_in: &str,
                           pass: bool,
                           shares_needed: usize,
                           secret_out: &str) -> Result<(), Box<dyn Error>> {
    let mut share_lists: Vec<Vec<BigInt>> = Vec::with_capacity(shares_needed);
    
    let file_in_paths = generate_share_file_paths(input_dir, stem, shares_needed);

    for i in 0..shares_needed {

        // Open one of the share files
        // If it cannot be read, print error information and abort.
        let mut share_file = match File::open(&file_in_paths[i]) {
            Ok(file) => file,
            Err(e) => {
                println!("Error reading in share file '{}': {}", &file_in_paths[i], e);
                println!("{}", RECONSTRUCT_ABORT);
                return Err(Box::new(e));

            }


        };

        let mut buf_8: [u8; 8] = [0; 8];

        // The first 8 bytes is going to be the number of shares in this file:
        share_file.read_exact(&mut buf_8)?; 
        let num_shares = u64::from_be_bytes(buf_8);

        // The current share list for this file
        let mut share_list: Vec<BigInt> = Vec::with_capacity(num_shares as usize);

        for _ in 0..num_shares {
            // The first 4 bytes (after the initial 8 bytes for the number of shares) is the number
            // of bytes for the next share
            let mut buf_4: [u8; 4] = [0; 4];
            (&mut share_file).read_exact(&mut buf_4)?;
            let bytes_for_next_share = u32::from_be_bytes(buf_4);


            // Preallocate the number of bytes for the share
            let mut share_bytes: Vec<u8> = Vec::with_capacity(bytes_for_next_share as usize); 

            // The next 'bytes_for_next_share' bytes will be the next share
            (&mut share_file).take(bytes_for_next_share as u64).read_to_end(&mut share_bytes)?;
            let share = BigInt::from_signed_bytes_be(share_bytes.as_slice());
            share_list.push(share);

        }

        // Push the generated share list into the list of share lists
        share_lists.push(share_list);
    }

    if pass { 
        let password = dialoguer::PasswordInput::new()
                                        .with_prompt("Password for shares")
                                        .with_confirmation("Confirm password", "Passwords don't match")
                                        .interact()?;

        share_lists = shuffle_share_lists(share_lists, password.as_ref(), ShuffleOp::ReverseShuffle);
    }

    // Now with the secret unshuffled, they should have x-values in the order of 1,2,3... etc. 
    // These need to be re-added for the reconstruction function
    // The +1 for x_value is needed because enumerate generates indices from 0..<len of iteration>
    // but our coefficients start at 1.
    let mut x_val_counter = 0;
    let share_lists: Vec<Vec<Point>> = share_lists
                     .into_iter()
                     .map(|share_list| {
                        *(&mut x_val_counter) = *(&x_val_counter) + 1;
                        share_list.into_iter()
                            .map(|y_val| {
                                Point::new(*(&x_val_counter), y_val)
                            }).collect()
                     }).collect();


    // Now get the prime from the file
    // If the prime file cannot be read, print error information and abort reconstruction
    let mut prime_in_file = match File::open(generate_prime_file_path(input_dir, prime_in)) {
        Ok(file) => file,
        Err(e) => {
            println!("Error reading prime input file: {}", e);
            println!("{}", RECONSTRUCT_ABORT);
            return Err(Box::new(e));
        }
    };

    // The first 4 bytes is the number of bytes for the prime.
    let mut buf_4: [u8; 4] = [0; 4];
    prime_in_file.read_exact(&mut buf_4)?;
    let prime_num_bytes = u32::from_be_bytes(buf_4);
    let mut prime_bytes: Vec<u8> = Vec::with_capacity(prime_num_bytes as usize);
    prime_in_file.take(prime_num_bytes as u64).read_to_end(&mut prime_bytes)?;
    let prime = BigInt::from_signed_bytes_be(prime_bytes.as_slice());
    

    // Now reconstruct the secret
    let reconstructed_secret = reconstruct_secrets_from_share_lists(share_lists,
                                                                    &prime,
                                                                    shares_needed)?;
   

    // Now we have the reconstructed secret, write it to a file
    let mut secret_out_file = File::create(secret_out)?;
    secret_out_file.write_all(reconstructed_secret.as_slice())?;
    Ok(())
}


#[derive(Debug)]
enum ValidPathType {
    ExistingFile, // The path leads to an existing file
    ExistingDir, // The path leads to an existing directory
    NonExisting, // No file or directory exists, but the path is valid
    Invalid // The path is not valid, file doesn't exist and it's parent directory doesn't exist
}


fn check_path(path: &str) -> ValidPathType {
    let path = Path::new(path);
    if path.exists() {
        if path.is_dir() {
            return ValidPathType::ExistingDir;
        }
        else {
            return ValidPathType::ExistingFile;
        }
    }
    else {
        let mut path_buf = path.to_path_buf();
        path_buf.pop();
        if path_buf.to_str().unwrap() == "" {
            // The path was a nonexsting file in the current directory, since popping it off the
            // buf led to an empty path
            return ValidPathType::NonExisting;
        }

        if path_buf.is_dir() {
            return ValidPathType::NonExisting;
        }
        else {
            // If the path doesn't exist and it's parent doesn't exist, it's not a valid path
            return ValidPathType::Invalid;
        }
    }
}


// Generates paths for the shares with in given dir with a given stem. 
// It is assumed that dir is a valid directory, no checks are done.
fn generate_share_file_paths(dir: &str, stem: &str, num_files: usize) -> Vec<String> {
    let mut path_buf = Path::new(dir).to_path_buf();
    let mut generated_paths: Vec<String> = Vec::with_capacity(num_files);

    for i in 0..num_files {
        path_buf.push(format!("{}.s{}", stem, i));
        (&mut generated_paths).push(String::from(path_buf.to_str().unwrap()));
        path_buf.pop();
    }

    generated_paths
}

// Generates the path for the prime file with the given dir and file path
fn generate_prime_file_path(dir: &str, prime_file: &str) -> String {
    let mut path_buf = Path::new(dir).to_path_buf();
    path_buf.push(prime_file);
    String::from(path_buf.to_str().unwrap())
}

