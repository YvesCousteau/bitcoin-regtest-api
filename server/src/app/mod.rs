use actix_web::{web, Scope};

mod regtest;

pub fn services() -> Scope {
    web::scope("").service(regtest::views::service())
}
