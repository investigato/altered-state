use crate::config::app::AppConfig;
#[derive(Clone, Debug)]
pub struct LdapOptions {
    pub domain: String,
    pub password: Option<String>,
    pub username: Option<String>,
    pub ldapfqdn: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub ldaps: bool,
    pub ldap_filter: Option<String>,
    pub kerberos: bool,
}

impl Default for LdapOptions {
    fn default() -> Self {
        Self {
            domain: "example.com".to_string(),
            ldapfqdn: "ldap.example.com".to_string(),
            password: None,
            username: None,
            ip: Some("127.0.0.1".to_string()),
            port: Some(389),
            ldaps: false,
            ldap_filter: Some("(objectClass=*)".to_string()),
            kerberos: true,
        }
    }
}

impl LdapOptions {
    pub fn generate_ldap_config(&self, config: &AppConfig, debug_mode: bool) -> LdapOptions {
        match debug_mode {
            true => LdapOptions {
                domain: config.domain.to_string(),
                ldapfqdn: config.hostname.to_string(),
                ip: Some(config.hostname.to_string()),
                port: Some(389),
                ldaps: false,
                ldap_filter: Some("(objectClass=*)".to_string()),
                username: Some("administrator".to_string()),
                password: Some("P@ssword123!".to_string()),
                kerberos: false,
            },
            false => LdapOptions {
                domain: config.domain.to_string(),
                ldapfqdn: config.hostname.to_string(),
                ip: Some("127.0.0.1".to_string()),
                port: Some(389),
                ldaps: false,
                ldap_filter: Some("(objectClass=*)".to_string()),
                username: None,
                password: None,
                kerberos: true,
            },
        }
    }
}
