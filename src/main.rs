use sss_rs::*;
use sss_rs::geometry::Point;
use num_bigint_dig::{BigInt, RandPrime};
use rand::rngs::StdRng;
use rand::FromEntropy;
use std::fs::File;
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

const SUBCOMMAND_RECONSTRUCT: &str = "reconstruct";
const ARG_OUTPUT_FILE: &str = "OUTPUT_FILE";
const ARG_PRIME_INPUT_FILE: &str = "PRIME_INPUT_FILE";
const ARG_INPUT_STEM: &str = "INPUT_STEM";

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
// TODO: Only one subcommand can be called from a given app/subcommand so no need to call match and
// check for each one, just use "subcommand" to return the sub command tuple and use that
fn run_with_args(args: &ArgMatches) {
    match args.subcommand() {
        (SUBCOMMAND_CREATE, Some(sub_matches)) => {
            /* I prefer this version, however it has some ref issues that the version
             * follwing this doesn't and if I could figure out a way to avoid that I would use this
             * version as it's much cleaner and I can add comments explaining how each argument is
             * handled and whatnot.
             * TODO: Look into the error and see if there's an easy way around it:
             * error[E0716]: temporary value dropped while borrowed
   --> src/main.rs:117:83
    |
117 |               let prime_out_file = sub_matches.value_of(ARG_OUTPUT_PRIME).unwrap_or(format!("{}.pri
me", 
    |  ___________________________________________________________________________________^
118 | |                                                     sub_matches.value_of(ARG_INPUT).unwrap()).as_
ref());
    | |                                                                                             ^    
      - temporary value is freed at the end of this statement
    | |_____________________________________________________________________________________________|
    |                                                                                               creat
es a temporary which is freed while still in use
...
126 |                                prime_out_file,
    |                                -------------- borrow later used here
    |
    = note: consider using a `let` binding to create a longer lived value
    = note: this error originates in a macro outside of the current crate (in Nightly builds, run with -Z
 external-macro-backtrace for more info)

             
            let file = sub_matches.value_of(ARG_INPUT).unwrap();
            let out_file_stem = sub_matches.value_of(ARG_OUTPUT_STEM)
                                    .unwrap_or(sub_matches.value_of(ARG_INPUT).unwrap());
            let shares_to_create = sub_matches.value_of(ARG_SHARES_TO_CREATE)
                                                        .unwrap().parse::<usize>().unwrap();

            let shares_needed = sub_matches.value_of(ARG_SHARES_NEEDED).unwrap_or(
                        sub_matches.value_of(ARG_SHARES_TO_CREATE).unwrap()).parse::<usize>().unwrap();
            let pass = sub_matches.is_present(ARG_WITH_PASSWORD);
            
            let prime_out_file = sub_matches.value_of(ARG_OUTPUT_PRIME).unwrap_or(format!("{}.prime", 
                                                    sub_matches.value_of(ARG_INPUT).unwrap()).as_ref());
            let prime_bits = 128;

            create_from_file(file,
                             out_file_stem,
                             shares_to_create,
                             shares_needed,
                             pass,
                             prime_out_file,
                             prime_bits).unwrap();
                             


            */
            create_from_file(
                sub_matches.value_of(ARG_INPUT).unwrap(),
                sub_matches.value_of(ARG_OUTPUT_STEM)
                        .unwrap_or(sub_matches.value_of(ARG_INPUT).unwrap()),
                sub_matches.value_of(ARG_SHARES_TO_CREATE).unwrap().parse::<usize>().unwrap(),
                sub_matches.value_of(ARG_SHARES_NEEDED).unwrap_or(
                        sub_matches.value_of(ARG_SHARES_TO_CREATE).unwrap()).parse::<usize>().unwrap(),
                sub_matches.is_present(ARG_WITH_PASSWORD),
                sub_matches.value_of(ARG_OUTPUT_PRIME).unwrap_or(format!("{}.prime", 
                                                    sub_matches.value_of(ARG_INPUT).unwrap()).as_ref()),
                128
                ).unwrap();
                
        },
        (SUBCOMMAND_RECONSTRUCT, Some(sub_matches)) => {
            reconstruct_from_shares(
                sub_matches.value_of(ARG_INPUT_STEM).unwrap(),
                sub_matches.value_of(ARG_PRIME_INPUT_FILE)
                    .unwrap_or(format!("{}.prime", 
                                       sub_matches.value_of(ARG_INPUT_STEM).unwrap()).as_ref()),
                sub_matches.is_present(ARG_WITH_PASSWORD),
                sub_matches.value_of(ARG_SHARES_NEEDED).unwrap().parse::<usize>().unwrap(),
                sub_matches.value_of(ARG_OUTPUT_FILE)
				.unwrap_or(format!("{}.out", sub_matches.value_of(ARG_INPUT_STEM).unwrap()).as_ref())
            ).unwrap();
        },
        _ => () // No subcommand was run
    } 



}



fn create_from_file(file: &str, 
                    out_file_stem: &str, 
                    shares_to_create: usize, 
                    shares_needed: usize, 
                    pass: bool, 
                    prime_out_file: &str, 
                    prime_bits: usize) -> Result<(), Box<dyn Error>> {

    let mut rand = StdRng::from_entropy();
    let prime: BigInt = rand.gen_prime(prime_bits).into();
    let co_max_bits = 128usize;
    let mut in_file = File::open(file)?;

    let mut secret = Vec::<u8>::new();
    in_file.read_to_end(&mut secret)?;

    let mut share_lists = create_share_lists_from_secrets(secret.as_slice(), &prime, shares_needed, 
                                                      shares_to_create, co_max_bits)?;
    // If a password is given, set share_lists to the shuffled share lists returned by the shuffle
    // op, else just set it to itself. Prevents an unecessary mut. TODO: Look into mutability,
    // maybe there's a better way to do this.
    if pass { 
        let password = dialoguer::PasswordInput::new()
                                        .with_prompt("Password for shares")
                                        .with_confirmation("Confirm password", "Passwords don't match")
                                        .interact()?;
        share_lists = shuffle_share_lists(share_lists, password.as_ref(), ShuffleOp::Shuffle);
    }



    // The format of the shares will be as so:
    //      First 8 bytes: The number of individual shares in the file
    //      Then for each share:
    //          4 bytes: The number of bytes for that given share
    //          N bytes: Determined by the previous 4 bytes, the share itself.
    let mut i = 0;
    for share_list in share_lists {
        let mut file_out = File::create(format!("{}.s{}", out_file_stem, i))?;
       
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

    // Write the prime out into the prime file
    let mut prime_out = File::create(prime_out_file)?;
    let prime_bytes = prime.to_signed_bytes_be();
    prime_out.write_all(&(prime_bytes.len() as u32).to_be_bytes())?;
    prime_out.write_all(prime_bytes.as_slice())?;


    Ok(())
}

// TODO: When multiple files are allowed, use a tuple with two options, one with the single stem and
// the other with a list of strs 
fn reconstruct_from_shares(stem: &str, 
                           prime_in: &str,
                           pass: bool,
                           shares_needed: usize,
                           secret_out: &str) -> Result<(), Box<dyn Error>> {
    let mut share_lists: Vec<Vec<BigInt>> = Vec::with_capacity(shares_needed);

    // This will be used for read operations when needing u32 and u64 values
    

    for i in 0..shares_needed {
        let mut share_file = File::open(format!("{}.s{}", stem, i))?;

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
    let mut prime_in_file = File::open(prime_in)?;

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




