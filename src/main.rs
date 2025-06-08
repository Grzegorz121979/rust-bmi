#![allow(unused)]

use std::fs::File;
use std::num::ParseFloatError;
use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, web, post};
use actix_web::web::{Form, Json};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use chrono::prelude::*;
use serde_json::json;
use std::io::Write;

#[derive(Deserialize, Serialize)]
struct UserData {
    name: String,
    weight: String,
    height: String,
}

async fn index() -> impl Responder {
    let tera = Tera::new("static/**/*").unwrap();
    let mut context = Context::new();
    let render = tera.render("index.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(render)
}

async fn calculate(form: Form<UserData>, tera: web::Data<Tera>) -> Result<impl Responder> {
    let weight = form.weight.trim().parse::<f64>();
    let height = form.height.trim().parse::<f64>();
    let mut context = Context::new();
   
    match (weight, height) {
        (Ok(w), Ok(h)) => {
            if w <= 0.0 || h <= 0.0 {
                context.insert("error", "Weight and height must be greater than zero.");
                let rendered = tera.render("index.html", &context).unwrap();
                Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
            }

            else if w > 500.0 || h > 300.0 {
                context.insert("error", "The provided values are outside the realistic range.");
                let rendered = tera.render("index.html", &context).unwrap();
                Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
            } 
            
            else {
                let h =  h / 100.0;
                let bmi = w / (h * h);
                let round_bmi = (bmi * 100.0).round() / 100.0;

                context.insert("result", &round_bmi);
                let rendered = tera.render("index.html", &context).unwrap();
                Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
            }
        }
        _ => {
            context.insert("error", "All fields should be filled in!");
            let rendered = tera.render("index.html", &context).unwrap();
            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
    }
}

async fn save_data(form: Form<UserData>) -> impl Responder {
    let local_date = Local::now().date_naive();
    let format_date = local_date.format("%d.%m.%Y").to_string();
    let json_data = json!({"name": form.name, "weight": form.weight, "date": format_date});

    match File::create("data.json") {
        Ok(mut file) => {
            if let Err(e) = write!(file, "{}", serde_json::to_string_pretty(&json_data).unwrap()) {
                return HttpResponse::InternalServerError().body(format!("{}", e));
            }
        },
        Err(e) => return HttpResponse::InternalServerError().body(format!("{}", e)),
    }
    
    HttpResponse::Ok().body("Data saved.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("static/**/*").unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
            .route("/sum", web::post().to(calculate))
            .route("/save", web::post().to(save_data))
            .service(Files::new("/", "./static"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
