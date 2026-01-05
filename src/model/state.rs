use sqlx::PgPool;
#[derive(Clone,Debug)]
pub struct AppState {
    pub db_pool : PgPool,
}

impl AppState {
    pub async fn new(db_pool: PgPool) -> Self {
        Self{db_pool}
    }
}