use std::convert::Infallible;
use std::net::SocketAddr;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use http::header;
use crate::db::{Database, Monitor, Change};
use tokio::net::TcpListener;

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }
    
    fn err(msg: &str) -> Self {
        Self { success: false, data: None, error: Some(msg.to_string()) }
    }
}

#[derive(Deserialize)]
pub struct AddMonitorRequest {
    pub url: String,
    pub interval_seconds: Option<u64>,
    pub name: Option<String>,
}

pub async fn run_api(db: Database, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("   API available at http://localhost:{}/api", port);
    
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let db = db.clone();
        
        // Spawn a task to handle the connection
        tokio::task::spawn(async move {
            let io = hyper_util::rt::tokio::TokioIo::new(stream);
            
            let service = service_fn(move |req| {
                let db = db.clone();
                async move {
                    handle_request(req, db).await
                }
            });
            
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error serving connection: {}", e);
            }
        });
    }
}

async fn handle_request(req: Request<hyper::body::Incoming>, db: Database) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    
    // Handle OPTIONS preflight
    if method == Method::OPTIONS {
        let mut response = Response::new(Full::new(Bytes::new()));
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, DELETE, OPTIONS".parse().unwrap());
        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type".parse().unwrap());
        return Ok(response);
    }
    
    let response = match (&method, path.as_str()) {
        // GET /api/monitors
        (&Method::GET, "/api/monitors") => {
            match db.get_monitors() {
                Ok(monitors) => {
                    let resp = ApiResponse::ok(monitors);
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
                Err(e) => {
                    let resp = ApiResponse::<Vec<Monitor>>::err(&e.to_string());
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
            }
        }
        
        // POST /api/monitors
        (&Method::POST, "/api/monitors") => {
            let resp = ApiResponse::ok(serde_json::json!({"message": "Use CLI to add monitors"}));
            Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
        }
        
        // GET /api/monitors/:id
        (&Method::GET, path) if path.starts_with("/api/monitors/") => {
            let id = path.trim_start_matches("/api/monitors/");
            match db.get_monitor(id) {
                Ok(Some(monitor)) => {
                    let resp = ApiResponse::ok(monitor);
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
                Ok(None) => {
                    let resp = ApiResponse::<Monitor>::err("Monitor not found");
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                        .unwrap()
                }
                Err(e) => {
                    let resp = ApiResponse::<Monitor>::err(&e.to_string());
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
            }
        }
        
        // DELETE /api/monitors/:id
        (&Method::DELETE, path) if path.starts_with("/api/monitors/") => {
            let id = path.trim_start_matches("/api/monitors/");
            match db.delete_monitor(id) {
                Ok(_) => {
                    let resp = ApiResponse::ok(id.to_string());
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
                Err(e) => {
                    let resp = ApiResponse::<String>::err(&e.to_string());
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
            }
        }
        
        // GET /api/changes/:monitor_id
        (&Method::GET, path) if path.starts_with("/api/changes/") => {
            let monitor_id = path.trim_start_matches("/api/changes/");
            match db.get_changes(Some(monitor_id)) {
                Ok(changes) => {
                    let resp = ApiResponse::ok(changes);
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
                Err(e) => {
                    let resp = ApiResponse::<Vec<Change>>::err(&e.to_string());
                    Response::new(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                }
            }
        }
        
        // 404
        _ => {
            let resp = ApiResponse::<()>::err("Not found");
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from(serde_json::to_string(&resp).unwrap())))
                .unwrap()
        }
    };
    
    // Add CORS headers
    let mut response = response;
    response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, DELETE, OPTIONS".parse().unwrap());
    response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type".parse().unwrap());
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response() {
        let ok = ApiResponse::ok(vec![1, 2, 3]);
        assert!(ok.success);
        assert!(ok.data.is_some());
        
        let err = ApiResponse::<String>::err("test error");
        assert!(!err.success);
        assert!(err.error.is_some());
    }
}
