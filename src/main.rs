use clap::Parser;
use std::io::prelude::*;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::process::exit;

fn main() -> io::Result<()> {

    let args = Args::parse();

    if vec!["s", "sender"].contains(&args.mode.as_str()) {
        send(args)
        
    }
    else if vec!["r", "receiver"].contains(&args.mode.as_str()) {
        receive(args)
    }
    else {
        if !args.silent {
            println!("Invalid mode!");
        }
        exit(1);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

struct Args {
    #[arg(short, long)]
    mode: String, // r/receiver or s/sender

    #[arg(short, long, default_value_t=String::from("0.0.0.0:53092"))]
    address: String, //Option<String> // in format of ip:port

    #[arg(short, long, default_value_t=8193)]
    bufsize: usize, // buffer size + terminator

    #[arg(short, long, default_value_t=false)]
    keep_open: bool, // should the listener keep listening after getting the first stream
    
    #[arg(short, long, default_value_t=false)]
    silent: bool, // should the functions print anything?

}

fn receive(args: Args) -> std::io::Result<()> {
    if !args.silent {
        println!("Receiving connections at {}", args.address);
    }
    let listener = TcpListener::bind(args.address)?;
    
    let mut firsttime = true;

    for stream in listener.incoming() {
        if firsttime {
            print!("{}[2J", 27 as char); // clears the screen by sending a control character according to this page: https://rosettacode.org/wiki/Terminal_control/Clear_the_screen
            firsttime = false;
        }
        handle_client(stream?, args.bufsize, args.keep_open, args.silent)
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, bufsize: usize, keep_open: bool, silent: bool) {
    let mut empty_vec = vec![0 as u8; bufsize+1];
    let mut buf = empty_vec.as_mut_slice();

    loop {
        match stream.read(&mut buf) {
            Ok(_) => {
                let mut stdout = io::stdout().lock();
                stdout.write_all(&buf).expect("Couldn't write to stdout");
                stream.shutdown(std::net::Shutdown::Both).expect("Couldn't shut down the connection");
                //buf = empty_vec.as_mut_slice();
            },
            Err(_) => {
                if !silent {
                    println!("An error occured!");
                }
                exit(1)
            }
        };

        if keep_open {
            break;
        }

        else {
            exit(0);
        }
    } 
    
}

fn send(args: Args) -> std::io::Result<()> {

    let mut buffer = Vec::<u8>::new();
    let mut stdin = io::stdin();

    stdin.read_to_end(&mut buffer)?;

    if !args.silent {
        println!("Sending message:\n{}", String::from_utf8_lossy(&buffer));
    }

    while !match TcpStream::connect(&args.address) {
        Ok(mut stream) => {

            stream.write(&buffer)?;
            //stream.flush(); 

            stream.shutdown(std::net::Shutdown::Both).expect("Couldn't shut down the connection");
            true
        },
        Err(_) => {
            if !args.silent {
                println!("An error occured while trying to connect to {}. Trying again in 3 seconds.", &args.address);
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
            false
        }
    } {}

    Ok(())
}