#[macro_use]
extern crate log;
extern crate argparse;
extern crate env_logger;
extern crate hyper;
extern crate mozprofile;
extern crate mozrunner;
extern crate regex;
extern crate rustc_serialize;
#[macro_use]
extern crate webdriver;

use std::borrow::ToOwned;
use std::env;
use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::str::from_utf8;

use argparse::{ArgumentParser, StoreTrue, Store};
use webdriver::server::start;

use marionette::{MarionetteHandler, BrowserLauncher, MarionetteSettings, extension_routes};

macro_rules! try_opt {
    ($expr:expr, $err_type:expr, $err_msg:expr) => (
        match $expr {
            Some(x) => x,
            None => return Err(WebDriverError::new($err_type, $err_msg)),
        }
    )
}

macro_rules! println_stderr {
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
}

mod marionette;

struct Options {
    binary: String,
    webdriver_host: String,
    webdriver_port: u16,
    marionette_port: u16,
    connect_existing: bool
}

fn parse_args() -> (Options, String) {
    let mut opts = Options {
        binary: "".to_owned(),
        webdriver_host: "127.0.0.1".to_owned(),
        webdriver_port: 4444u16,
        marionette_port: 2828u16,
        connect_existing: false
    };
    let usage: String;

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("WebDriver to marionette proxy.");
        parser.refer(&mut opts.binary)
            .add_option(&["-b", "--binary"], Store,
                        "Path to the Firefox binary");
        parser.refer(&mut opts.webdriver_host)
            .add_option(&["--webdriver-host"], Store,
                        "Host to run webdriver server on");
        parser.refer(&mut opts.webdriver_port)
            .add_option(&["--webdriver-port"], Store,
                        "Port to run webdriver on");
        parser.refer(&mut opts.marionette_port)
            .add_option(&["--marionette-port"], Store,
                        "Port to run marionette on");
        parser.refer(&mut opts.connect_existing)
            .add_option(&["--connect-existing"], StoreTrue,
                        "Connect to an existing firefox process");
        parser.parse_args_or_exit();

        let mut buf = Vec::<u8>::new();
        parser.print_help(&program(), &mut buf).unwrap();
        usage = from_utf8(&buf[..]).unwrap().to_string();
    }

    (opts, usage)
}

fn program() -> String {
    if let Some(a) = env::args().nth(0) {
        a
    } else {
        "wires".to_string()
    }
}

fn error(usage: &str, message: &str) -> ! {
    println_stderr!("{}: error: {}", program(), message);
    println_stderr!("");
    println_stderr!("{}", usage);
    exit(1)
}

fn main() {
    env_logger::init().unwrap();

    let (opts, usage) = parse_args();
    if opts.binary == "" && !opts.connect_existing {
        error(&usage, "must supply a binary path or --connect-existing");
    }

    let host = &opts.webdriver_host[..];
    let port = opts.webdriver_port;
    let addr = Ipv4Addr::from_str(host).map(
        |x| SocketAddr::V4(SocketAddrV4::new(x, port))).unwrap_or_else(
        |_| {
            error(&usage, "invalid host address");
        }
        );

    let launcher = if opts.connect_existing {
        BrowserLauncher::None
    } else {
        BrowserLauncher::BinaryLauncher(Path::new(&opts.binary).to_path_buf())
    };

    let settings = MarionetteSettings::new(opts.marionette_port, launcher);

    //TODO: what if binary isn't a valid path?
    start(addr, MarionetteHandler::new(settings), extension_routes());
}
