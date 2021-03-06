extern crate clipboard;
extern crate byteorder;
extern crate argparse;
extern crate encoding;

use clipboard::ClipboardContext;
use std::net::{TcpListener,TcpStream};
use std::io::{Error, ErrorKind, Result  };
use std::thread;
use byteorder::{ReadBytesExt, WriteBytesExt,  LittleEndian};
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;
use argparse::{ArgumentParser, Store,StoreTrue};
use std::io::prelude::*;
use std::io;

use encoding::{Encoding, EncoderTrap, DecoderTrap,CodecError};
use encoding::all::ISO_8859_1;

struct Config {
    skip_client : bool,
    port : u16,
    localip : String,
    outsideip : String
}


fn run_reading(stream : &mut TcpStream,content : Sender<String>) -> Result<()>{
    loop {
        println!("Waiting for a data");
        let length : u32 = try!(stream.read_u32::<LittleEndian>());
        if length == 0{
            // ignore empty
            continue;
        }
        println!("Got the data size: {}", length);
        // let mut data = vec![0u8;length as usize];
        let mut data = vec![0u8;0];
        println!("");
        try!(stream.take(length as u64).read_to_end(&mut data));
        for v in data.iter(){
            print!("{} ",v);
        }
        let decoded = match ISO_8859_1.decode(&data[..],DecoderTrap::Strict){
            Ok(s) => s,
            Err(_) => "Error decoding".to_owned()
        };
        let _ = content.send(decoded);
    }
}


fn run_sync(stream : &mut TcpStream) -> Result<()>{
    let mut ctx = match ClipboardContext::new(){
        Ok(c) => c,
        Err(_) => return Err(Error::new(ErrorKind::Other,"Failed to create clipboard context"))
    };
    // println!("{}", ctx.get_contents().unwrap());
    //
    // ctx.set_contents("the new content".to_owned()).unwrap();
    let mut current_content = match ctx.get_contents(){
        Ok(s) => s,
        Err(_) => "".to_owned()
    };
    let (wx, rx) = channel::<String>();
    // // create thread which will read the stream for changes from outside, and apply changes to this system
    //
    let mut reading = stream.try_clone().unwrap();
    thread::spawn(move||{run_reading(&mut reading,wx)});
    let mut writer = stream.try_clone().unwrap();

    println!("initial clipboard value is: {}",current_content );

    let mut counter = 0;

    loop{
        thread::sleep(Duration::new(1,0));
        if counter % 64 == 0{
            println!("");
        }
        counter = counter + 1;
        print!("-");
        io::stdout().flush().ok().expect("Could not flush stdout");
        match ctx.get_contents(){
            Ok(s) => {

                if s != current_content{
                    println!("A new content, sending it to the other side: {}!={}",s,current_content);
                    current_content = s.clone();
                    let data = match ISO_8859_1.encode(&s[..],EncoderTrap::Strict){
                        Ok(d) => d,
                        Err(e) => {
                            println!("{}", e);vec!()}
                    };
                    for v in data.iter(){
                        print!("{} ",v );
                    }
                    println!("");
                    try!(writer.write_u32::<LittleEndian>(data.len() as u32));
                    try!(writer.write(&data[..]));
                }
                match rx.try_recv(){
                    Ok(v) =>{
                        println!("Received a new value for cb: {}", v );
                        if v != current_content{
                            current_content = v.clone();
                            match ctx.set_contents(v){
                                Ok(_) => println!("The new content has been set."),
                                Err(_) => println!("Failed to set the new content.")
                            }
                        }
                    }
                    Err(_) => ()
                }
            }
            Err(_) => ()
        };

    }
}

fn try_run_client(config : &Config) -> Result<()>{
    println!("Trying to connect to {}:{}",config.outsideip, config.port );
    let mut stream = try!(TcpStream::connect((&config.outsideip as &str,config.port)));
    run_sync(&mut stream)
}


fn run_server(config : &Config) -> Result<()> {
    println!("Creating server on {}:{}" , config.localip,config.port );
    let listener = try!(TcpListener::bind((&config.localip as &str,config.port)));
    println!("Waiting for new connections");

    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                thread::spawn(move||{
                    println!("connected");
                    run_sync(&mut stream.try_clone().unwrap())
                });
            }
            Err(e) => {
                println!("Connection failed {}",e );
            }
        }
    }

    drop(listener);
    Ok(())
}



fn main() {
    let mut config = Config{
        skip_client : false,
        port: 24011,
        localip : "127.0.0.1".to_owned(),
        outsideip : "127.0.0.1".to_owned()
    };
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Synchronise clipboard content between two computers.");
        ap.refer(&mut config.skip_client)
            .add_option(&["-s","--skip_client"],StoreTrue,"Skip connecting to client and create server right away.");
        ap.refer(&mut config.port)
            .add_option(&["-p","--port"],Store,"Port address");
        ap.refer(&mut config.localip)
            .add_option(&["-l","--local"],Store,"Local ip address");
        ap.refer(&mut config.outsideip)
            .add_option(&["-o","--outside"],Store,"Outside ip address");
        ap.parse_args_or_exit();
    }
    println!("local: {}, outside: {}, port: {}",config.localip, config.outsideip, config.port );
    let _ = if config.skip_client{
        println!("Skiping connecting to client");
        run_server(&config)
    }else {match try_run_client(&config){
        Err(_) => {
            println!("Could not connect to server, creating own.");
            run_server(&config)},
        _ => Ok(())
    }};
}
