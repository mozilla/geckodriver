extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate hyper;
extern crate mozprofile;
extern crate mozrunner;
extern crate regex;
extern crate rustc_serialize;
#[macro_use]
extern crate slog;
extern crate slog_atomic;
extern crate slog_stdlog;
extern crate zip;
extern crate webdriver;

#[macro_use]
extern crate log;

pub mod logging;

macro_rules! try_opt {
    ($expr:expr, $err_type:expr, $err_msg:expr) => ({
        match $expr {
            Some(x) => x,
            None => return Err(WebDriverError::new($err_type, $err_msg))
        }
    })
}

// TODO(ato): Split marionette.rs up further
mod marionette;
pub use marionette::{extension_routes, FirefoxOptions, MarionetteHandler, MarionetteSettings};
