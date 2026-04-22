use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::error::RunError;
use crate::format::print_response;
use crate::parse::RequestBlock;

pub fn execute_request(req: &RequestBlock, http_file: &str) -> Result<(), RunError> {
    let (save_filename, headers): (Option<String>, Vec<_>) = {
        let mut save = None;
        let filtered = req
            .headers
            .iter()
            .filter(|(name, value)| {
                if name.eq_ignore_ascii_case("x-file") {
                    save = Some(value.clone());
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        (save, filtered)
    };

    let mut request = ureq::request(&req.method, &req.url);
    for (name, value) in &headers {
        request = request.set(name, value);
    }

    let start = Instant::now();
    let response = if let Some(body) = &req.body {
        request.send_string(body)
    } else {
        request.call()
    };
    let elapsed = start.elapsed();

    match response {
        Ok(resp) => {
            let body = print_response(resp, req, elapsed);
            if let Some(ref filename) = save_filename {
                let path = save_body(&body, http_file, filename);
                println!("\nSaved to: {}", path);
            }
            Ok(())
        }
        Err(ureq::Error::Status(_, resp)) => {
            let body = print_response(resp, req, elapsed);
            if let Some(ref filename) = save_filename {
                let path = save_body(&body, http_file, filename);
                println!("\nSaved to: {}", path);
            }
            Ok(())
        }
        Err(ureq::Error::Transport(e)) => Err(RunError::Transport(e.to_string())),
    }
}

fn save_body(body: &str, http_file: &str, filename: &str) -> String {
    let dir = Path::new(http_file)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let path = dir.join(filename);
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = f.write_all(body.as_bytes());
    }
    path.to_string_lossy().into_owned()
}
