mod storage;
mod model;
mod sql_value;

use anyhow::Result;
use crate::sql_value::SqlVal;

fn main() -> Result<()> {
    storage::init_db()?;
    let test = storage::get_by_col("name", SqlVal::Text("test"))?;
    assert_eq!(test, None);

    Ok(())
}
