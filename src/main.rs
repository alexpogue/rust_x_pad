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
            out = Box::new(File::create(&args[4]).expect("Unable to create file"));
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
