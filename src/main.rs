//external crates
extern crate clap;
extern crate hyper;

//usings
use clap::{Arg, App, SubCommand};
use tokio::io::{stdout, AsyncWriteExt as _};
use hyper::Client;
use hyper::body::{HttpBody, Body};
use hyper::Response;
use hyper::Request;
use hyper_tls::HttpsConnector;
use std::io::Read;

#[derive(Debug, PartialEq)]
enum HttpMethod{
    Post,
    Get,
    Put,
    Patch,
    Delete    
}

#[derive(Debug)]
struct AppConfiguration{
    http_method: HttpMethod,
    request_body: String,
    request_body_as_file: String,
    request_content_type: String,
    request_url: String
}

impl AppConfiguration{
    fn default() -> AppConfiguration{
        return AppConfiguration{
            http_method: HttpMethod::Post,
            request_body: "".parse().unwrap(),
            request_body_as_file: "".parse().unwrap(),
            request_content_type: "".parse().unwrap(),
            request_url: "".parse().unwrap()
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    //read command line arguments
    let matches = App::new("Rust Man")
                    .version("1.0")
                    .author("Robert Reyes")
                    .about("HTTP Utility")
                    .arg(Arg::with_name("requesturl")
                        .long("requesturl")
                        .value_name("REQUEST_URL")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name("httpmethod")
                        .long("httpmethod")
                        .value_name("HTTP_METHOD")
                        .required(true)
                        .takes_value(true))
                    .arg(Arg::with_name("requestbody")
                        .long("requestbody")
                        .value_name("REQUEST_BODY")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name("requestfile")
                        .long("requestfile")
                        .value_name("REQUEST_FILE")
                        .required(false)
                        .takes_value(true))
                    .arg(Arg::with_name("requestcontenttype")
                        .long("requestcontenttype")
                        .value_name("REQUEST_BODY_AS_FILE")
                        .required(false)
                        .takes_value(true))
                    .get_matches();


    //populate app configuration
    let mut app_config = AppConfiguration::default();
    if let Some(provided_http_method) = matches.value_of("httpmethod"){
        if provided_http_method == "post"{
            app_config.http_method = HttpMethod::Post
        }
        else if provided_http_method == "put"{
            app_config.http_method = HttpMethod::Put
        }
        else if provided_http_method == "patch"{
            app_config.http_method = HttpMethod::Patch
        }
        else if provided_http_method == "delete"{
            app_config.http_method = HttpMethod::Delete
        }
        else if provided_http_method == "get"{
            app_config.http_method = HttpMethod::Get
        }
    }

    //check if the http method requires a request body and request content type
    if app_config.http_method != HttpMethod::Get && app_config.http_method != HttpMethod::Delete{
        //check for raw request first
        let provided_request_body = matches.value_of("requestbody");

        //if raw request body was not provided, check for file
        if provided_request_body == None{
            let provided_request_file = matches.value_of("requestfile");

            //panic because we need request in some form
            if provided_request_file == None{
                panic!("Request body was not provided");
            }
            
            app_config.request_body_as_file = provided_request_file.unwrap().parse().unwrap();
        }
        else{
            app_config.request_body = provided_request_body.unwrap().parse().unwrap();
        }

        //get request content type
        let provided_request_content_type = matches.value_of("requestcontenttype");
        if provided_request_content_type == None{
            panic!("Request content type was not provided");
        }

        app_config.request_content_type = provided_request_content_type.unwrap().parse().unwrap();
    }

    //setup request uri
    let provided_request_url = matches.value_of("requesturl");
    if provided_request_url == None{
        panic!("Request url was not provided");
    }
    app_config.request_url = provided_request_url.unwrap().parse().unwrap();

    //DEBUG remove
    println!("Here are the values you provided. HTTP Method: {:?}, Request Body: {}, Request Body as File: {}, Request Content Type: {}, Request URL: {}",
        app_config.http_method,
        app_config.request_body,
        app_config.request_body_as_file,
        app_config.request_content_type,
        app_config.request_url
    );

    //setup SSL/TLS http client
    let https = HttpsConnector::new();
    let http_client = Client::builder().build::<_, hyper::Body>(https);

    //make appropriate http call
    let mut resp: Response<Body> = Response::default();
    if app_config.http_method == HttpMethod::Get{
        resp = http_client.get(app_config.request_url.parse().unwrap()).await?;
    }
    else if app_config.http_method == HttpMethod::Post{
        let request = Request::builder().header("content-type", app_config.request_content_type.clone()).method("POST").uri(app_config.request_url.clone()).body(Body::from(app_config.request_body)).expect("request builder");
        resp = http_client.request(request).await?;
    }
    else if app_config.http_method == HttpMethod::Post{
        let request = Request::builder().method("PUT").uri(app_config.request_url.clone()).body(Body::from(app_config.request_body)).expect("request builder");
        resp = http_client.request(request).await?;
    }
    else if app_config.http_method == HttpMethod::Post{
        let request = Request::delete(app_config.request_url.clone()).body(Body::from("")).unwrap();
        resp = http_client.request(request).await?;
    }

    //print response
    println!("RESPONSE STATUS CODE: {}", resp.status());
    println!("RESPONSE BODY");
    for chunk in resp.body_mut().data().await{
        stdout().write_all(&chunk?).await?;
    }

    return Ok(());
}
