use std::env;
use std::fs;
use std::io::Write;
use std::io;
use rand::Rng;

fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("not enough arguments"); 
    }

    if args[1].eq("-g") {
        if args.len() < 3 {
            panic!("not enough arguments. provide length of one-time-pad key");
        }
        let key_len_mb = &args[2].parse::<usize>().expect("Failed to parse int argument to `-g` flag");

        let mut out: Box<dyn Write> = Box::new(io::stdout());
        if args.len() >= 5 && args[3] == "-o" {
            out = Box::new(fs::File::create(&args[4]).expect("Unable to create file"));
        }

        const BYTES_PER_MEGABYTE:usize = 1024 * 1024; // 1024 bytes -> kb, 1024 kb -> mb
        let key_len_bytes = key_len_mb * BYTES_PER_MEGABYTE;
        let mut rng = rand::thread_rng();

        const MAX_CHUNK_SIZE_BYTES: usize = BYTES_PER_MEGABYTE/2;
        let mut chunk = [0u8; MAX_CHUNK_SIZE_BYTES];

        let mut bytes_written = 0;
        while bytes_written < key_len_bytes {
            let bytes_to_write = (key_len_bytes - bytes_written).clamp(0,MAX_CHUNK_SIZE_BYTES);
            rng.fill(&mut chunk[..bytes_to_write]);
            bytes_written += out.write(&chunk).expect("Output Error. Try piping to a file.");
        }
        out.flush().unwrap();
    } else {
        if args.len() < 3 {
            panic!("not enough arguments"); 
        }
        let key = fs::read(&args[1]).unwrap();
        let message = fs::read(&args[2]).unwrap();

        let result: Vec<u8> = message.iter()
                .zip(key[..message.len()].iter())
                .map(|(x, y)| *x ^ *y)
                .collect();

        let mut out = fs::OpenOptions::new().write(true).truncate(true).open(&args[2]).unwrap();
        out.write(&result).expect("unable to write file");
        out.flush().unwrap();
        fs::remove_file(&args[1]).unwrap();
    }
}