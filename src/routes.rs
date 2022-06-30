
#[get("/ping")]
pub fn ping() -> &'static str {
    "ok"
}
