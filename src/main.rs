use std::env;
use std::process;
use std::fs::File;
use std::io::ErrorKind;
use std::io::{BufReader, Read, Write};
use std::io::Error;
use std::io;
use rand::Rng;

fn main() {
    let args:Vec<String> = env::args().collect();

    if args[1].eq("-g") {
        let num_megabytes_in_key_str = &args[2];
        let num_megabytes_in_key = num_megabytes_in_key_str.parse::<usize>().expect("Failed to parse int argument to `-g` flag");
        const BYTES_PER_MEGABYTE:usize = 1024 * 1024; // 1024 bytes -> kb, 1024 kb -> mb
        let mut num_bytes_in_key = num_megabytes_in_key * BYTES_PER_MEGABYTE;
        let mut rng = rand::thread_rng();

        const MAX_CHUNK_SIZE_BYTES: usize = 256 * BYTES_PER_MEGABYTE;
        let mut chunk = Vec::with_capacity(MAX_CHUNK_SIZE_BYTES);
        while num_bytes_in_key > 0 {
            let num_bytes_in_chunk = std::cmp::min(num_bytes_in_key, MAX_CHUNK_SIZE_BYTES);
            for _ in 0..num_bytes_in_chunk {
                chunk.push(rng.gen());
            }
            io::stdout().write_all(&chunk).expect("Write error");
            num_bytes_in_key -= num_bytes_in_chunk;
        }
    } else {
        let file_name = &args[1];
        let f = File::open(file_name).expect("Unable to open file");
        let mut br = BufReader::new(f);
        
        let mut stdin = io::stdin();

        let mut key_buf: [u8; 128] = [0; 128];
        let mut message_buf: [u8; 128] = [0; 128];
        let mut key_n = 1;
        let mut message_n = 1;

        while key_n != 0 && message_n != 0 {
            message_n = match read_and_retry_upon_interrupt(&mut stdin, &mut message_buf, 0, 5) {
                Ok(n) => n,
                Err(e) => print_file_read_error_and_abort(file_name, e),
            };
            if message_n == 0 {
                break;
            }
            key_n = match read_and_retry_upon_interrupt(&mut br, &mut key_buf, message_n.try_into().unwrap(), 5) {
                Ok(n) => n,
                Err(e) => print_file_read_error_and_abort(file_name, e),
            };
            
            if key_n < message_n {
                eprintln!("Error: message is longer than key");
                process::abort();
            }

            let result: Vec<u8> = message_buf.iter()
                .zip(key_buf.iter())
                .map(|(x, y)| *x ^ *y)
                .collect();

            io::stdout().write(&result[0..message_n]).expect("Write error");
        }
    }
}

fn read_and_retry_upon_interrupt(read_from:&mut dyn Read, read_to:&mut[u8], limit:u64, retry_times:u8) -> Result<usize, Error> {
    let limited:&mut dyn Read = &mut read_from.take(limit);
    let limited_read_from:&mut dyn Read = if limit > 0 {
         limited
    } else {
        read_from
    };
    for _ in 0..retry_times-1 {
        match limited_read_from.read(read_to) {
            Ok(n) => return Ok(n),
            Err(e) => match e.kind() {
                ErrorKind::Interrupted => continue,
                _ => return Err(e),
            },
        };
    };
    return limited_read_from.read(read_to);
}

fn print_file_read_error_and_abort(file_name:&String, e:std::io::Error) -> ! {
    eprintln!("Error reading from message file {}, error = {}", file_name, e);
    process::abort();
}