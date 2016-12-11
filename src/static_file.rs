extern crate url;

use std::path::{Path, PathBuf};
use hyper::server::{Request, Response};
use hyper::method::Method::{Get, Head};
use hyper::uri::RequestUri::AbsolutePath;
use {Middleware, FileSender};

pub struct StaticFile {
    fs_root: String,
    url_root: String,
}

impl<Ctx> Middleware<Ctx> for StaticFile {
    fn handle<'a, 'b, 'c>(&self, req: Request<'a, 'b>, res: Response<'c>, context: Ctx)
        -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {

        self.serve_file(req, res, context)
    }
}

impl StaticFile {
    pub fn new(url_root: String, fs_root: String) -> StaticFile {
        StaticFile {
            fs_root: fs_root,
            url_root: url_root,
        }
    }

    fn serve_file<'a, 'b, 'c, Ctx>(&self, req: Request<'a, 'b>, res: Response<'c>, context: Ctx)
        -> Option<(Request<'a, 'b>, Response<'c>, Ctx)> {

        if req.method == Get || req.method == Head {
            if let AbsolutePath(ref url_path) = req.uri {
                if let Some(file_path) = url_to_file_path(url_path) {
                    if file_path.starts_with(&self.url_root) {
                        //TODO better unwraps
                        let file_path = file_path.strip_prefix(&self.url_root).unwrap();
                        let path = Path::new(&self.fs_root).canonicalize().unwrap().join(file_path);

                        println!("static path {:?}", path);

                        if path.is_file() {
                            res.send_file(path.to_str().unwrap().to_string());
                            return None;
                        }
                    }
                }
            }
        }

        Some((req, res, context))
    }
}


fn url_to_file_path(url_path: &String) -> Option<PathBuf> {
    let url_struct = match url::Url::parse(&*format!("http://dummy.com{}", url_path)) {
        Ok(val) => val,
        Err(_) => return None,
    };

    let path = url_struct.path();
    let path = Path::new(path);

    if path.ends_with("/") {
        path.join("index.html");
    }

    Some(path.to_path_buf())
}

