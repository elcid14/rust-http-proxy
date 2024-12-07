use std::{collections::HashMap, error::Error, f32::consts::E, fmt, io::{Read, Write}, net::{IpAddr, SocketAddr, TcpListener, TcpStream}, thread};
use serde::{ Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use local_ip_address::local_ip;


#[derive(Serialize, Deserialize, Debug)]
struct HttpRequest{
    headers: HashMap<String,String>,
    body: String
}

enum HttpError {
    Message(String)
}


impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Message(msg) => write!(f, "Malformed request: {}", msg),
        }
    }
}




fn parse_incoming_reqest(mut incoming_request: &TcpStream, request_ip_add:SocketAddr) -> Result<HttpRequest, HttpError> {
    println!("Incoming request ip:{}", request_ip_add);
    
    // let message = String::from("POST / HTTP/\r\nHost: 172.22.242.155:8000\r\nAccept-Encoding: gzip, deflate\r\nAccept: */*\r\nConnection: keep-alive\r\nContent-Length: 0\r\nUser-Agent: HTTPie/3.2.1\r\n\r\n");


    let response: &str = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\ntest\n";
    incoming_request.write_all(response.as_bytes()).unwrap_or(println!("Error sending response request"));
    incoming_request.flush().unwrap_or(println!("Error terminating request threat"));

    // Define buffer size
    let mut buffer: [u8; 1024] = [0u8; 1024];
    // Read incoming TCP Stream
    let bytes_read: usize = incoming_request.read(&mut buffer).unwrap();
    // Read from request
    let message: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Incoming request:{:?}", message);
    // Create vector to collect the request
    let header_lines: Vec<&str> =  message.split("\r\n\r\n").collect();

    if header_lines.is_empty(){
        return Err(HttpError::Message("Empty request".to_string()));
    }

    
    // Defines the headers
    let headers: &&str = header_lines.get(0).unwrap_or(&"");
    let body: &&str = header_lines.get(1).unwrap_or(&"");
    // Define the hasmap to store in header field for HttpRequest struct
    let mut headers_content_map: HashMap<String, String> = HashMap::new();

    // 
    let mut headers_method_and_version: std::str::Lines<'_> = headers.lines();
    if let Some(request) = headers_method_and_version.next() {
        let method: Vec<&str> = request.split_whitespace().collect();
        if method.len() == 3{
            headers_content_map.insert("Method".to_string(), method[0].to_string());
            headers_content_map.insert("Path".to_string(), method[1].to_string());
            headers_content_map.insert("Version".to_string(), method[2].to_string());
        }
    }



    for line in headers.lines(){
        if let Some((key,value)) = line.split_once(":"){
            headers_content_map.insert(key.to_string(), value.to_string());
        }
    }



    let response: HttpRequest =  HttpRequest {
        headers: headers_content_map,
        body: body.to_string()
    };

    
    Ok(response)
    }




fn main() -> std::io::Result<()> {
    let data: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let server_ip: IpAddr = local_ip().unwrap();
    let port:i32 = 8000;
    let address: String = format!("{server_ip}:{port}");

    let  listener: TcpListener = TcpListener::bind(address)?;
    

    println!("Server running on: {:?}", server_ip);
 
    for stream in listener.incoming(){
        let data: Arc<Mutex<i32>> = Arc::clone(&data);
        match stream {
            Ok(stream   ) => {
                thread::spawn(move ||  {
                    let request_ip: std::net::SocketAddr = stream.peer_addr().unwrap();
                    let thread_id: thread::ThreadId = thread::current().id();
                    println!("Thread id:{:?}", thread_id);
                    // let mut data: std::sync::MutexGuard<'_, i32> = data.lock().unwrap();
                    let parsed_request: Result<_, Result<(HttpRequest), std::io::Error>> = match parse_incoming_reqest(&stream,request_ip){
                    Err(e) => Err(e),
                    Ok(parsed_request) => Ok(parsed_request),
                   };
                   println!("Parsed request:{:?}", parsed_request);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }   
    }
    
    Ok(())
}
