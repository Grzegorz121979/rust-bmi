#![allow(unused)]

use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, web};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use chrono::prelude::*;

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

async fn calculate_sum(form: web::Form<UserData>, tera: web::Data<Tera>) -> Result<impl Responder> {
    let weight = form.weight.trim().parse::<f64>();
    let height = form.height.trim().parse::<f64>();
    
    match (weight, height) {
        (Ok(w), Ok(h)) => {
            let mut context = Context::new();
            let h =  h / 100.0;
            let bmi = w / (h * h);
            let round_bmi = (bmi * 100.0).round() / 100.0; 
            context.insert("result", &round_bmi);

            let rendered = tera.render("index.html", &context).unwrap();
            
            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
        _ => {
            let mut context = Context::new();
            context.insert("error", "All fields should be filled!");

            let rendered = tera.render("index.html", &context).unwrap();

            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
    }
}

async fn save(form: web::Form<UserData>, tera: web::Data<Tera>) -> impl Responder {
    let local_date = Local::now().date_naive();
    let format_date = local_date.format("%d.%m.%Y");
    HttpResponse::Ok().body(format!("{}", format_date))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("static/**/*").unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
            .route("/sum", web::post().to(calculate_sum))
            .route("/save", web::post().to(save))
            .service(Files::new("/", "./static"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
