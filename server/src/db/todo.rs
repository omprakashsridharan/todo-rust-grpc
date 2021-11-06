// #[derive(Debug, FromRow, Clone)]
pub struct TodoItemDb {
    pub id: u32,
    pub description: String,
    pub status: i32,
}
