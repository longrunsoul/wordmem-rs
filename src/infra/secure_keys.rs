use anyhow::Result;
use keyring;

const KEYRING_SERVICE: &str = "wordmem.lrs";

#[derive(Debug, PartialEq)]
pub struct SyncKeys {
    pub imap_server: String,
    pub smtp_server: String,
    pub email: String,
    pub password: String,
}

impl SyncKeys {
    fn try_get_keys() -> Result<SyncKeys> {
        Ok(SyncKeys {
            imap_server: keyring::Entry::new(KEYRING_SERVICE, "imap_server").get_password()?,
            smtp_server: keyring::Entry::new(KEYRING_SERVICE, "smtp_server").get_password()?,
            email: keyring::Entry::new(KEYRING_SERVICE, "email").get_password()?,
            password: keyring::Entry::new(KEYRING_SERVICE, "password").get_password()?,
        })
    }

    pub fn clear_keys() -> Result<()> {
        let _ = keyring::Entry::new(KEYRING_SERVICE, "imap_server").delete_password();
        let _ = keyring::Entry::new(KEYRING_SERVICE, "smtp_server").delete_password();
        let _ = keyring::Entry::new(KEYRING_SERVICE, "email").delete_password();
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
        keyring::Entry::new(KEYRING_SERVICE, "imap_server").set_password(&self.imap_server)?;
        keyring::Entry::new(KEYRING_SERVICE, "smtp_server").set_password(&self.smtp_server)?;
        keyring::Entry::new(KEYRING_SERVICE, "email").set_password(&self.email)?;
        keyring::Entry::new(KEYRING_SERVICE, "password").set_password(&self.password)?;

        Ok(())
    }
}