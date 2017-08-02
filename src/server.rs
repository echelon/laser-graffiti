
use iron::Handler;
use std::io::Read;
use serde_json;
use iron::Iron;
use iron::IronResult;
use iron::IronError;
use iron::Request;
use iron::Response;
use iron::status;
use router::Router;


struct DrawHandler {
}

impl DrawHandler {
  pub fn new() -> DrawHandler {
    DrawHandler {}
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
  pub x: i16,
  pub y: i16,
}

#[derive(Debug, Serialize, Deserialize)]
struct DrawRequest {
  pub points: Vec<Point>,
}

impl Handler for DrawHandler {
  fn handle(&self, request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();

    let _ = request.body.read_to_string(&mut body)
        .map_err(|e| IronError::new(e, "Read error occurred"))?;

    println!("Request Body: {}", body);

    let draw_request : DrawRequest = serde_json::from_str(&body)
        .map_err(|e| IronError::new(e, "Parsing error occurred"))?;

    println!("Request Json: {:?}", draw_request);

    let mut response = Response::new();
    Ok(response)
  }
}

pub fn start_http_server() {
  let mut router = Router::new();

  router.get("/", |_: &mut Request| {
    Ok(Response::with((status::Ok, "Index")))
  }, "index");

  router.post("/draw", DrawHandler::new(), "draw");

  // TODO: Parameters for port and hostname
  let host = "localhost:8888";
  println!("Launching server on: {}", host);
  Iron::new(router).http(host).expect("Server could not start");
}
