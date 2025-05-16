use actix_web::{ web, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use serde::{Deserialize,Serialize};
use std::process::{Command};
use std::sync::{ Mutex};

#[derive(Deserialize,Serialize)]

struct CommandRequest {
    command: String,
    password: String,
}

#[derive(Serialize)]

struct  CommandResponse {
    output: String,
    error: String,
}

async  fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Termideus....")
}

async  fn termideus_page() -> impl Responder {
    let html = include_str!("static/terminal.html");
    HttpResponse::Ok().content_type("text/html").body(html)

}

async fn execute_command(req_body: web::Json<CommandRequest>, data : web::Data<AppState>) -> impl Responder {

    let  terminal_pwd = data.password.lock().unwrap();


    if req_body.password != *terminal_pwd {
        return  HttpResponse::Forbidden().json(CommandResponse {
            output: String::new(),
            error : "You did no provide proper access to Termadeus...".to_string(),
        });
    }

    if req_body.command.contains("../") || req_body.command.contains("./") || req_body.command.contains("/.") {
        return  HttpResponse::Forbidden().json(CommandResponse {
            output: String::new(),
            error : "Command Rejected..You are not allowed to access the directory".to_string(),
        });
    }

    let blocked_cmds =  ["rm -rf /", "rm -rf /*", "> /etc/passwd", "chmod -R 777"];
    if blocked_cmds.iter().any(|&cmd| req_body.command.contains(cmd)) {
        return  HttpResponse::Forbidden().json(CommandResponse {
            output : String::new(),
            error : "Command execution was denied : Destructive to TermiDeus.".to_string(),
        });
    }

    let cmd_result = Command::new("sh")
    .arg("-c")
    .arg(&req_body.command)
    .output();

    match cmd_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            HttpResponse::Ok().json(CommandResponse{
                output : stdout,
                error: stderr,
            })
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(CommandResponse {
                output: String::new(),
                error: format!("deusTerminal failed to execute {}", e),
            })
        }
    }
}

struct  AppState {
    password : Mutex<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {


    println!("....Starting TerminaDeus as port: 8080");


    let app_state = web::Data::new( AppState {
        password : Mutex::new("akane7000b!".to_string()),
    });
    
    HttpServer::new( move || {
        App::new()
        .app_data(app_state.clone())
        .service(Files::new("static", "./static"))
        .route("/", web::get().to(index))
        .route("/terminal",web::get().to(termideus_page))
        .route("/api/execute", web::post().to(execute_command))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}