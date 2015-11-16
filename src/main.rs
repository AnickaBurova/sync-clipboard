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
        let length : u32 = try!(stream.read_u32::<LittleEndian>());
        let mut data = vec![0u8;length as usize];
        try!(stream.take(length as u64).read_to_end(&mut data));

        match String::from_utf8(data){
            Ok(s) => {
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

fn run_sync(stream : TcpStream) -> Result<()>{
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
    thread::spawn(move||{run_reading(&mut stream.try_clone().unwrap(),wx)});

    loop{
        thread::sleep(Duration::new(1,0));
        match ctx.get_contents(){
            Ok(s) => {
                if s != current_content{
                    let data = s.as_bytes();
                    // stream.write(data);
                }
            }
            Err(_) => ()
        };

    }
}

fn try_run_client() -> Result<()>{
    let mut stream = try!(TcpStream::connect(&HOST));
    run_sync(stream)
}


fn run_server() -> Result<()> {
    let listener = try!(TcpListener::bind(HOST));

    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                thread::spawn(move||{
                    println!("connected");
                    run_sync(stream)
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
