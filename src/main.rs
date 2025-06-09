#![allow(unused)]

use actix_files::Files;
use actix_web::web::Form;
use actix_web::{App, HttpResponse, HttpServer, Responder, Result, web};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::num::ParseFloatError;
use tera::{Context, Tera};

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

fn bmi_result(form: &Form<UserData>) -> Result<f64, ParseFloatError> {
    let weight = form.weight.trim().parse::<f64>()?;
    let height = form.height.trim().parse::<f64>()?;
    
    if weight == 0.0 || height == 0.0 {
        Ok(0.0)
    } else if weight == 400.0 || height == 300.0 {
        Ok(0.0)
    } else {
        let height_meters = height / 100.0;
        let bmi = weight / (height_meters * height_meters);
        let round_bmi = (bmi * 100.0).round() / 100.0;

        Ok(round_bmi)
    }
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
            } else if w > 500.0 || h > 300.0 {
                context.insert(
                    "error",
                    "The provided values are outside the realistic range.",
                );
                let rendered = tera.render("index.html", &context).unwrap();
                Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
            } else {
                let h = h / 100.0;
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

async fn save_data(form: Form<UserData>, tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    let local_date = Local::now().date_naive();
    let format_date = local_date.format("%d.%m.%Y").to_string();
    let mut bmi_f64 = bmi_result(&form).unwrap();
    let json_data =
        json!({"name": form.name, "weight": form.weight, "date": format_date, "bmi": bmi_f64});

    match File::create("data.json") {
        Ok(mut file) => {
            if let Err(e) = write!(
                file,
                "{}",
                serde_json::to_string_pretty(&json_data).unwrap()
            ) {
                return HttpResponse::InternalServerError()
                    .body(format!("Error writing to file: {}", e));
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error creating file: {}", e));
        }
    }
    context.insert("error", "Data saved!");
    let rendered = tera.render("index.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
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
