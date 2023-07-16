use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

#[allow(dead_code)]
struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}

const MAX_PORT: u16 = 65535;

impl Arguments {
    // Here [&'static str] is used to handle errors
    // CMD : cargo run -- -t <threads> <IP_ADDR>
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments");
        } else if args.len() > 4 {
            return Err("Too many arguments");
        }
        let f = args[1].clone();
        // Case : cargo run -- <IP_ADDR>
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                flag: String::from(""),
                ipaddr,
                threads: 4,
            });
        } else {
            // Case : cargo run -- -h | cargo run -- --help
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("--help") && args.len() == 2 {
                println!(
                    "Syntax :
cargo run -- <option>

Options : 
-h | --help : help
-t : threads : default -> 4

Examples :
* cargo run -- -h
* cargo run -- -t 10 127.0.0.1
                "
                );
                return Err("help");
            } else if flag.contains("-h") || flag.contains("--help") {
                // Case : cargo run -- <any> -h | cargo run -- <any> --help
                return Err("Too many arguments");
            } else if flag.contains("-t") {
                // Case : cargo run -- -t <option : threads> <IP_ADDR>
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("Not a valid IP_ADDRESS"),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("Failed to parse the number of threads"),
                };
                return Ok(Arguments {
                    threads,
                    flag,
                    ipaddr,
                });
            } else {
                return Err("Invalid Syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                println!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }

        if (MAX_PORT - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguments : {}", program, err);
            process::exit(0);
        }
    });

    let num_threads = arguments.threads;
    // tx: transmitter
    // rx : reciever
    let (tx, rx) = channel();

    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, arguments.ipaddr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("port {} is open", v);
    }
}
