#[derive(Clone, Debug)]
pub enum GitlabRef {
    Repo(String),
    Branch(String, String),
}

impl From<&str> for GitlabRef {
    fn from(item: &str) -> Self {
        // TODO: weak error handling
        match item.find('@') {
            Some(offset) => {
                GitlabRef::Branch(item[..offset].to_string(), item[offset + 1..].to_string())
            }
            None => GitlabRef::Repo(item.to_string()),
        }
    }
}
