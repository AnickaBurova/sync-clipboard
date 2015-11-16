extern crate clipboard;
extern crate byteorder;

use clipboard::ClipboardContext;
use std::net::{TcpListener,TcpStream};
use std::io::{Error, ErrorKind, Result, Read,Write};
use std::thread;
use byteorder::{ReadBytesExt, WriteBytesExt,  LittleEndian};
use std::sync::mpsc::{Sender, Receiver,channel};
use std::str;
use std::time::Duration;


static HOST:&'static str = "127.0.0.1:24011";


fn run_reading(stream : &mut TcpStream,content : Sender<String>) -> Result<()>{
    loop {
        println!("Waiting for a data");
        let length : u32 = try!(stream.read_u32::<LittleEndian>());
        println!("Got the data size: {}", length);
        let mut data = vec![0u8;length as usize];
        try!(stream.take(length as u64).read_to_end(&mut data));
        match String::from_utf8(data){
            Ok(s) => {
                println!("Received new content from the other side: {}", s);
                let _ = content.send(s);
            }
            Err(_) => {
                println!("Failed to convert received content to string.");
            }
        };
        // try!(str::from_utf8(&data));
        // content.send(Some(other_content));
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

    loop{
        thread::sleep(Duration::new(1,0));
        println!("Checking for new content!");
        match ctx.get_contents(){
            Ok(s) => {

                if s != current_content{
                    println!("A new content, sending it to the other side: {}!={}",s,current_content);
                    current_content = s.clone();
                    let data = s.as_bytes();
                    try!(writer.write_u32::<LittleEndian>(data.len() as u32));
                    try!(writer.write(data));
                }
                match rx.try_recv(){
                    Ok(v) =>{
                        println!("Received a new value for cb: {}", v );
                        if v != current_content{
                            current_content = v.clone();
                            match ctx.set_contents(v){
                                Ok(_) => println!("Seting the new value as clipboard content."),
                                Err(_) => println!("Failde to set the new value as clipboard content.")
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

fn try_run_client() -> Result<()>{
    let mut stream = try!(TcpStream::connect(&HOST));
    run_sync(&mut stream)
}


fn run_server() -> Result<()> {
    let listener = try!(TcpListener::bind(HOST));

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
    let res = match try_run_client(){
        Err(_) => {
            println!("Could not connect to server, creating own.");
            run_server()},
        _ => Ok(())
    };

    match res {
        Ok(_) => println!("done"),
        Err(e) => println!("Failed: {}", e),
    }

    // let mut ctx = ClipboardContext::new().unwrap();
    // println!("{}", ctx.get_contents().unwrap());
    //
    // ctx.set_contents("the new content".to_owned()).unwrap();
}
