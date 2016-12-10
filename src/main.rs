extern crate hyper;

use std::fs;
use std::io::ErrorKind::NotFound;
use hyper::server::{Server, Request, Response};
use hyper::method::Method::{Get, Head};
use hyper::uri::RequestUri::AbsolutePath;

//type Middleware = for<'a, 'b, 'c> fn(req: Request<'a, 'b>, res: Response<'c>) -> Option<(Request<'a, 'b>, Response<'c>)>;

macro_rules! endIfNone {
    ($e:expr) => {
        match $e {
            Some(val) => val,
            None => return,
        }
    }
}



/// # Design comments
///
/// ### Middleware
///
/// - create a Trait for this that is commonly implemented to Functions and Closures
///
/// the basic is this
///
/// ```
/// type MiddlewareFn = for<'a, 'b, 'c, T> fn(req: Request<'a, 'b>, res: Response<'c>, context: T) ->
///   Option<(Request<'a, 'b>, Response<'c>, T)>;
/// ```
///
/// which basically translates simply to this
///
/// Request -> Response -> Context -> Option<(Request, Response, Context)>
///
///
/// This means that middlewares, in the bottom are simply functions that take a Request and a
/// Response, which should be regular Hyper Request, Response plus a Context that will be helpful
/// to store data in the middleware processing chain, i.e. session data, et al
///
/// Of course this should be only the type for Function Middlewares,
/// Struct and Custom type middleware should be also available very similarly to how nickel and
/// iron do it.
///
/// Additionally it'll be nice to have a utility function to that takes a vector of middleware
/// and runs then in order. Something like this
///
/// ```
/// fn process_middleware(req, res, context, m: Vec<Middleware>) -> Option<(Request, Response, Context)> {
///     for middleware in m {
///          let (req, res, context) = match middleware(req, res, context) {
///             Some(val) => val,
///             _ => return None,
///          }
///     }
/// }
/// ```
///
/// This utilitarian function will be the only entry point to the _framework_ and it should be run
/// from inside the handle function that we use as Hyper handle
///
/// ```
/// fn handler(req: Request, res: Response) {
///     let middleware = ...;
///     let context = ...;
///     let (req, res, context) = endIfNone!(process_middleware(req, res, context, middleware));
///     // regain native handle of req, res and context and do whatever you please
///     // if anything at all
/// }
/// ```
///
/// Note that in a sense , process_middleware is also a middleware :)
///
///
/// ### Helper methods
///
/// Helper methods such as `send_file`, `render` et al, should be implemented
/// as a Trait for the type Response.
///
/// ```
/// trait SendFile { ... }
/// trait Render { ... }
/// //etc
/// ```
/// In this sense the user will be able to
///
/// ```
/// res.render(myTemplate);
/// ```
///
/// Things to have in consideration
///
/// - We should provide implementation for this traits, as much as we can
/// - This traits should be able to be defined in a custom base by the user
/// - this will enable to provide a standard interface for defining custom rendering methods
/// with any template engine that the user wants :)
/// - figure out how to use effectively trait objects 
///
///
/// ### Static file
///
/// - We should use a standard url parsing module
/// - It should sit ontop of the above
/// - it should be safe, not letting users to access unwanted files
///
/// ### Router
///
/// TODO
///
/// ### Templates
///
/// TODO
///
fn main() {
    Server::http("127.0.0.1:8080").unwrap().handle(handler).unwrap();
}

fn logger<'a, 'b, 'c, Ctx>(req: Request<'a, 'b>, res: Response<'c>, context: Ctx) -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {
    println!("LOG {:?}", req.uri);
    Some((req, res, context))
}

//fn staticFile(fsRoot: String, httpRoot: String) -> Middleware {
    //fn middleware<'a, 'b, 'c>(req: Request<'a, 'b>, res: Response<'c>) -> Option<(Request<'a, 'b>, Response<'c>)> {
        //println!("LOG {:?}", req.uri);
        //match req.method {
            //Get | Head => {
                //match req.uri {

                    //AbsolutePath(ref path) => {
                        //let path = path.splitn(2, '?').next().unwrap();
                        //let path = match path {
                            //"/" => "index.html",
                            //path => &path[1..],
                        //};

                         //match fs::metadata(&path) {
                            //Ok(ref attr) if attr.is_file() => {
                                //res.send(path.as_bytes());
                            //},
                            //Err(ref e) if e.kind() != NotFound => println!("Error getting metadata for file '{:?}': {:?}", path, e),
                            //_ => {},
                        //}
                    //},
                    //_ => return Some((req, res)),
                //}
            //},
            //_ => return Some((req, res)),
        //}

        //None
    //}

    //return middleware;
//}

fn router<'a, 'b, 'c, Ctx>(req: Request<'a, 'b>, res: Response<'c>, context: Ctx) -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {
    let s = format!("Hi mofo {}", req.uri);
    res.send(s.as_bytes()).unwrap();
    None
}

// Each middleware should return the request so it's easy to
// follow the stream of middleware
fn handler(req: Request, res: Response) {

    let context: Vec<String> = vec!();
    let l = &logger;
    let r = &router;
    let middleware: Vec<&Middleware<Vec<String>>> = vec![l, r];
    let (req, res, context) = endIfNone!(process_middleware(middleware, req, res, context));
}

trait Middleware<Ctx> {
    fn handle<'a, 'b, 'c>(&self, req: Request<'a, 'b>, res: Response<'c>, context: Ctx)
        -> Option<(Request<'a, 'b>, Response<'c>, Ctx)>;
}

impl<F, Ctx> Middleware<Ctx> for F
    where F: for<'a, 'b, 'c> Fn(Request<'a, 'b>, Response<'c>, Ctx)
        -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {

    fn handle<'a, 'b, 'c>(&self, req: Request<'a, 'b>, res: Response<'c>, context: Ctx)
        -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {
        (*self)(req, res, context)
    }
}


fn process_middleware<'a, 'b, 'c, Ctx>(middleware_vec: Vec<&Middleware<Ctx>>, req: Request<'a, 'b>, res: Response<'c>, context: Ctx)
    -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {

    let mut value = Some((req, res, context));

    for m in middleware_vec {
        if let Some((req, res, context)) = value {
            value = m.handle(req, res, context);
        }
    }

    value
}

//TODO
//- static file server
//- Router
//- templates/rendering
//
//
//
