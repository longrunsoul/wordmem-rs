use anyhow::Result;
use keyring;

const KEYRING_SERVICE: &str = "wordmem.lrs";

#[derive(Debug, PartialEq)]
pub struct SyncKeys {
    pub email_domain: String,
    pub username: String,
    pub password: String,
}

impl SyncKeys {
    fn try_get_keys() -> Result<SyncKeys> {
        Ok(SyncKeys {
            email_domain: keyring::Entry::new(KEYRING_SERVICE, "email_domain").get_password()?,
            username: keyring::Entry::new(KEYRING_SERVICE, "username").get_password()?,
            password: keyring::Entry::new(KEYRING_SERVICE, "password").get_password()?,
        })
    }

    pub fn clear_keys() -> Result<()> {
        let _ = keyring::Entry::new(KEYRING_SERVICE, "email_domain").delete_password();
        let _ = keyring::Entry::new(KEYRING_SERVICE, "username").delete_password();
        let _ = keyring::Entry::new(KEYRING_SERVICE, "password").delete_password();

        Ok(())
    }

    pub fn get_keys() -> Result<Option<SyncKeys>> {
        let keys = SyncKeys::try_get_keys();
        if keys.is_err() {
            SyncKeys::clear_keys()?;
            return Ok(None)
        }

        let keys = keys.unwrap();
        Ok(Some(keys))
    }

    pub fn set_keys(&self) -> Result<()> {
        keyring::Entry::new(KEYRING_SERVICE, "email_domain").set_password(&self.email_domain)?;
        keyring::Entry::new(KEYRING_SERVICE, "username").set_password(&self.username)?;
        keyring::Entry::new(KEYRING_SERVICE, "password").set_password(&self.password)?;

        Ok(())
    }
}