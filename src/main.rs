mod infra;
mod word_taker;
mod revisit_planner;

use anyhow::Result;

use infra::*;

fn main() -> Result<()> {
    storage::init_db()?;
    let test = storage::get_by_col("name", SqlVal::Text("test"))?;
    assert_eq!(test, None);

    Ok(())
}
