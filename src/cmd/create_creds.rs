use crate::auth::create::ssh::setup_ssh_auth;
use crate::config::BGitConfig;

pub fn create_creds(_config: BGitConfig) {
    setup_ssh_auth();
}
