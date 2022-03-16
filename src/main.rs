mod storage;

use anyhow::Result;

fn main() -> Result<()> {
    storage::init_db()?;

    Ok(())
}
