#[derive(Debug)]
pub struct PathEntry {
    pub id: Option<i64>,
    pub path: String,
    pub last_used: i64,
    pub freq: i64,
}

#[derive(Debug)]
pub struct TagEntry {
    pub id: Option<i64>,
    pub path_id: i64,
    pub tag: String,
} 