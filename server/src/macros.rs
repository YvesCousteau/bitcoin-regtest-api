#[macro_export]
macro_rules! export_routes {
    ($($x:tt),+ $(,)?) => {
        pub fn service() -> ($($x,)+) {
            actix_web::services!($($x,)+)
        }
    }
}
