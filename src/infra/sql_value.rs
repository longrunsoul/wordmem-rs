use std::fmt::{Display, Formatter};

pub enum SqlVal<'a> {
    #[allow(dead_code)]
    Null,
    Integer(i64),
    #[allow(dead_code)]
    Real(f64),
    Text(&'a str),
    #[allow(dead_code)]
    Blob(&'a [u8]),
}

impl<'a> SqlVal<'a> {
    pub fn repr(&self) -> String {
        match *self {
            SqlVal::Null => "NULL".to_string(),
            SqlVal::Integer(i) => i.to_string(),
            SqlVal::Real(f) => f.to_string(),
            SqlVal::Text(s) => format!("'{}'", prepare_sql_value(s)),
            SqlVal::Blob(b) => format!("x'{}'", hex::encode(b)),
        }
    }
}

impl<'a> Display for SqlVal<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.repr())?;
        Ok(())
    }
}

fn prepare_sql_value(s: &str) -> String {
    let mut result = s.to_string();

    // anti-injection
    result = result.replace('\'', "");

    result
}
