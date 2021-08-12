use conch_parser::ast::Redirect;

#[derive(Debug, PartialEq)]
pub enum State {
    Default,
    Condition,
}

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) transaction: Vec<String>,
    pub(crate) sql: String,
    pub(crate) redirect: Option<Redirect<String>>,
    pub(crate) state: State,
}

impl Default for Intermediate {
    fn default() -> Self {
        Intermediate {
            transaction: Vec::new(),
            sql: "".to_string(),
            redirect: None,
            state: State::Default,
        }
    }
}

impl Intermediate {
    pub fn get_sql(&mut self) -> String {
        if !self.sql.is_empty() {
            self.transaction.push(self.sql.to_string());
            self.sql.clear();
        }
        let mut sql: String = String::new();
        if !self.transaction.is_empty() {
            sql = self.transaction.join("; ");
            self.transaction.clear();
            sql.push(';');
        }
        sql
    }
}
