use netdoor::NetDoor;
use ansi_escapes::*;

use std::io::Write;
use std::net::*;

fn bounce(socket: TcpStream) {
    let mut conn = NetDoor::connect(socket, None);
    match conn.negotiate_winsize() {
        Ok(true) => (),
        Ok(false) => eprintln!("no winsize"),
        Err(e) => eprintln!("no winsize: {}", e),
    }
    let termok = conn.negotiate_cbreak()
        .and_then(|_| conn.negotiate_noecho())
        .and_then(|_| conn.negotiate_ansi());
    match termok {
        Ok(true) => (),
        e => {
            let mut socket = conn.into_inner();
            eprintln!("cannot set up terminal: {:?}", e);
            socket.write(
    b"Your telnet client cannot be put in \r\n\
      no-echo single-character ANSI mode\r\n\
      as needed to play the game. Apologies.\r\n").unwrap();
            return;
        }
    }
    conn.set_timeout(Some(100));
    let width = conn.width.unwrap();
    let height = conn.height.unwrap();

    macro_rules! cprint {
        ($fmt:expr, $($arg:expr),+) => {
            conn.write_all(format!($fmt, $($arg),*).as_bytes()).unwrap();
        };
    }

    cprint!("{}", ClearScreen);
    loop {
        cprint!("{}", CursorTo::AbsoluteXY((height / 2) as u16, (width / 2) as u16));
        if conn.read().unwrap().is_some() {
            cprint!("{}", CursorTo::AbsoluteXY((height - 1) as u16, 0));
            return;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12999").unwrap();
    loop {
        let (socket, _) = listener.accept().unwrap();
        let _ = std::thread::spawn(move || bounce(socket));
    }
}
