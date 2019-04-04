extern crate rocket;
extern crate serde_json;

/*
#[cfg(test)]
mod tests;

use std::{io, env};
use std::fs::File;

use rocket::{Request, Handler, Route, Data, Catcher};
use rocket::http::{Status, RawStr};
use rocket::response::{self, Responder, status::Custom};
use rocket::handler::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::http::Method::*;

fn forward<'r>(_req: &'r Request, data: Data) -> Outcome<'r> {
    Outcome::forward(data)
}

fn hi<'r>(req: &'r Request, _: Data) -> Outcome<'r> {
    Outcome::from(req, "Hello!")
}

fn name<'a>(req: &'a Request, _: Data) -> Outcome<'a> {
    let param = req.get_param::<&'a RawStr>(0)
        .and_then(|res| res.ok())
        .unwrap_or("unnamed".into());

    Outcome::from(req, param.as_str())
}

fn echo_url<'r>(req: &'r Request, _: Data) -> Outcome<'r> {
    let param = req.get_param::<&RawStr>(1)
        .and_then(|res| res.ok())
        .into_outcome(Status::BadRequest)?;

    Outcome::from(req, RawStr::from_str(param).url_decode())
}

fn upload<'r>(req: &'r Request, data: Data) -> Outcome<'r> {
    if !req.content_type().map_or(false, |ct| ct.is_plain()) {
        println!("    => Content-Type of upload must be text/plain. Ignoring.");
        return Outcome::failure(Status::BadRequest);
    }

    let file = File::create(env::temp_dir().join("upload.txt"));
    if let Ok(mut file) = file {
        if let Ok(n) = io::copy(&mut data.open(), &mut file) {
            return Outcome::from(req, format!("OK: {} bytes uploaded.", n));
        }

        println!("    => Failed copying.");
        Outcome::failure(Status::InternalServerError)
    } else {
        println!("    => Couldn't open file: {:?}", file.unwrap_err());
        Outcome::failure(Status::InternalServerError)
    }
}

fn get_upload<'r>(req: &'r Request, _: Data) -> Outcome<'r> {
    Outcome::from(req, File::open(env::temp_dir().join("upload.txt")).ok())
}

fn not_found_handler<'r>(req: &'r Request) -> response::Result<'r> {
    let res = Custom(Status::NotFound, format!("Couldn't find: {}", req.uri()));
    res.respond_to(req)
}

#[derive(Clone)]
struct CustomHandler {
    data: &'static str
}

impl CustomHandler {
    fn new(data: &'static str) -> Vec<Route> {
        vec![Route::new(Get, "/<id>", Self { data })]
    }
    fn new_post(data: &'static str) -> Vec<Route> {
        vec![Route::new(Post, "/", Self { data })]
    }
}

impl Handler for CustomHandler {
    fn handle<'r>(&self, req: &'r Request, data: Data) -> Outcome<'r> {
        let id = req.get_param::<&RawStr>(0)
            .and_then(|res| res.ok())
            .or_forward(data)?;

        println!("{:?}", req);

        Outcome::from(req, format!("{} - {}", self.data, id))
    }
}

fn rocket() -> rocket::Rocket {
    let always_forward = Route::ranked(1, Get, "/", forward);
    let hello = Route::ranked(2, Get, "/", hi);

    let echo = Route::new(Get, "/echo/<str>", echo_url);
    let name = Route::new(Get, "/<name>", name);
    let post_upload = Route::new(Post, "/", upload);
    let get_upload = Route::new(Get, "/", get_upload);

    let not_found_catcher = Catcher::new(404, not_found_handler);

    rocket::ignite()
        .mount("/", vec![always_forward, hello, echo])
        .mount("/upload", vec![get_upload, post_upload])
        .mount("/hello", vec![name.clone()])
        .mount("/hi", vec![name])
        .mount("/custom", CustomHandler::new("some data here"))
        .mount("/custom", CustomHandler::new_post("some data here"))
        .register(vec![not_found_catcher])
}

fn main() {
    rocket().launch();
}
*/

use rocket::{Request, Handler, Route, Data, Catcher};
use rocket::http::{Status, RawStr};
use rocket::response::{self, Responder, status::Custom};
use rocket::handler::Outcome;
use rocket::http::Method::*;
use std::io::Write;
use std::io::Result;

use serde_json::Value;

//https://api.rocket.rs/rocket/config/

macro_rules! get_param {
    ($i: ident, $e: expr) => {
        match $i.get_param::<&RawStr>($e) {
            Some(a) => {
                match a {
                    Ok(b) =>  b,
                    Err(_) => return Outcome::failure(Status::BadRequest)
                }
            },
            None => return Outcome::failure(Status::BadRequest),
        };
    }
}

struct StringWriter {
    /// String from body of request
    data: String,
    /// Body size limit
    size_limit: usize
}

/*
impl StringWriter {
    fn new(body_size_limit: u64) -> StringWriter {
        StringWriter {
            data: String::new(),
            body_size_limit
        }
    }

    fn read(&mut self, data: &Data) {
        println!("{}", self.body_size_limit);

        let mut buffer: &[u8];
        let mut new_data = String::new();
        let mut peek_complete = false;

        while !peek_complete {
            buffer = data.peek();
            let s = String::from_utf8_lossy(buffer);

            new_data.push_str(&s);
            peek_complete = data.peek_complete();
        }


        self.data = new_data;
    }

    fn get(&self) -> String {
        self.data.clone()
    }
}
*/

impl StringWriter {
    fn new(size_limit: usize) -> StringWriter {
        StringWriter {
            data: String::new(),
            size_limit
        }
    }

    fn get(&self) -> String {
        self.data.clone()
    }
}

impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {

        if self.data.len() + buf.len() > self.size_limit {
            println!("Lossing my religion !!!");

            let new_size = self.size_limit - self.data.len();

            let buf = &buf[..new_size];
            let s = String::from_utf8_lossy(buf);

            self.data.push_str(&s);

            return Ok(buf.len());
        }

        let s = String::from_utf8_lossy(buf);

        self.data.push_str(&s);

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Clone)]
struct GetHandler {
    // Put here database connection
    /// Body size limit
    body_size_limit: u64
}

impl GetHandler {
    fn new(body_size_limit: u64) -> Vec<Route> {
        vec![
            Route::new(Get, "/<account>/<collection>/<id>", Self {body_size_limit} ),
            // Create
            Route::new(Post, "/<account>/<collection>", Self {body_size_limit} ),
            // Update
            Route::new(Put, "/<account>/<collection>/<id>", Self {body_size_limit} ),
            Route::new(Delete, "/<account>/<collection>/<id>", Self {body_size_limit} ),
            Route::new(Patch, "/<account>/<collection>/<id>", Self {body_size_limit} )
        ]
    }

    fn extract_body(&self, data: Data) -> String {
        let mut s = StringWriter::new(self.body_size_limit as usize);

        data.stream_to(&mut s);

        s.get()
    }

    fn extract_json(&self, data: &str) -> std::result::Result<Value, ()> {
        match serde_json::from_str::<Value>(data) {
            Ok(v) => Ok(v),
            Err(e) => {
                // TODO Log
                Err(())
            }
        }
    }

    fn manage_request<'r>(&self, account: &str, connection: &str, id: &str, json: &Value, req: &'r Request) -> Outcome<'r> {
        Outcome::from(req, "Hello, world!")
    }
    /*
    fn new_post() -> Vec<Route> {
        vec![Route::new(Post, "/<account>/<collection>", Self)]
    }
    */
}

impl Handler for GetHandler {
    fn handle<'r>(&self, req: &'r Request, data: Data) -> Outcome<'r> {
        match req.content_type() {
            Some(c) => {
                if c.is_json() {
                    let json_text = self.extract_body(data);

                    match self.extract_json(&json_text) {
                        Ok(json) => {
                            let account = get_param!(req, 0);
                            let collection_name = get_param!(req, 1);
                            let id = get_param!(req, 2);

                            self.manage_request(account, collection_name, id, &json, req)
                        },
                        Err(_) => Outcome::failure(Status::BadRequest)
                    }
                } else {
                    Outcome::failure(Status::BadRequest)
                }
            },
            None => Outcome::failure(Status::BadRequest)
        }
    }
}

fn not_found_handler<'r>(req: &'r Request) -> response::Result<'r> {
    let res = Custom(Status::NotFound, format!("Sorry your path not found: {}", req.uri()));
    res.respond_to(req)
}


// curl --header "Content-Type: application/json" --request POST --data '{"username":"xyz","password":"xyz"}'  http://localhost:8000/echo/eee/rrr

fn main() {
    let not_found_catcher = Catcher::new(404, not_found_handler);

    let rocket = rocket::ignite();

    let body_size_limit = rocket.config().limits.get("json").unwrap_or(64000);

    println!("{:?}", rocket.config().limits.get("json"));

    rocket.mount("/", GetHandler::new(body_size_limit))
        .register(vec![not_found_catcher])
        .launch();
}