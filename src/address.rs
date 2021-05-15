use std::fmt;

use rand::seq::SliceRandom;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::{config::*, lookup::TxtLookup};

use crate::os::AppRegistry;

const TXT_ATTRIBUTE_NAME: &str = "tel";
const TEL_NUMBER_DELIMITER: &str = ";";
const EXTRA_DATA_DELIMITER: &str = "._tel.";

#[derive(Clone)]
pub struct DomainAddress {
    // dns address with extra data stripped
    addr: String,
    // extra dns path (if exists)
    extra_data: Option<String>,
    // list of resolved phone numbers (could be empty)
    resolved: Option<Vec<String>>,
}

impl fmt::Display for DomainAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> ", self.addr)?;

        write!(
            f,
            "{}",
            match &self.resolved {
                Some(resolved) => {
                    if resolved.is_empty() {
                        "-".to_owned()
                    } else {
                        resolved.join(TEL_NUMBER_DELIMITER)
                    }
                }
                None => "[unresolved]".to_owned(),
            }
        )
    }
}

impl DomainAddress {
    fn new(addr: &str) -> Self {
        let (addr, extra_data) =
            if let Some((extra_data, addr_clean)) = addr.split_once(EXTRA_DATA_DELIMITER) {
                (addr_clean.to_owned(), Some(extra_data.to_owned()))
            } else {
                (addr.to_owned(), None)
            };

        Self {
            addr,
            extra_data,
            resolved: None,
        }
    }

    // raw DNS address with extra path included
    pub fn raw_addr(&self) -> String {
        if let Some(extra) = &self.extra_data {
            format!("{}{}{}", extra, EXTRA_DATA_DELIMITER, self.addr)
        } else {
            self.addr.clone()
        }
    }

    // try to get general website info from OS
    pub fn general_info(&self, app_registry: &AppRegistry) -> Option<String> {
        app_registry.get_general_info(&self.addr)
    }

    // try to get extra info about subaddress from OS
    pub fn fetch_extra_info(&self, app_registry: &AppRegistry) -> Option<String> {
        if let Some(extra) = &self.extra_data {
            app_registry.get_extra_info(&self.addr, &extra)
        } else {
            None
        }
    }

    // get and store list of phone numbers associated with this DNS address
    pub fn resolve(&mut self, refresh: bool) -> Result<Vec<String>, String> {
        if !refresh {
            if let Some(resolved) = &self.resolved {
                return Ok(resolved.clone());
            }
        }

        let txts = self.get_txt()?;

        let mut resolved = vec![];

        for txt in txts {
            let txt = txt.to_string();
            // RFC 1464
            if let Some((attribute_name, attribute_value)) = txt.split_once('=') {
                if attribute_name != TXT_ATTRIBUTE_NAME {
                    continue;
                }

                resolved = attribute_value
                    .split(TEL_NUMBER_DELIMITER)
                    .map(|s| s.to_owned())
                    .collect();
            } else {
                continue;
            }
        }

        self.resolved = Some(resolved.clone());

        Ok(resolved)
    }

    fn get_txt(&self) -> Result<TxtLookup, String> {
        // TODO: reuse resolver?
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

        resolver
            .txt_lookup(self.raw_addr())
            .map_err(|e| format!("cannot resolve TXT: {}", e))
    }
}

#[derive(Clone)]
pub enum Address {
    PhoneNumber(String),
    DomainName(DomainAddress),
}

impl Address {
    pub fn new(addr: &str) -> Result<Self, String> {
        if addr.contains('.') {
            Ok(Self::DomainName(DomainAddress::new(addr)))
        } else if addr.starts_with('+') {
            Ok(Self::PhoneNumber(addr.to_owned()))
        } else {
            Err("not a valid domain name or phone number".to_owned())
        }
    }

    // raw address
    pub fn raw_addr(&self) -> String {
        match self {
            Self::PhoneNumber(addr) => addr.clone(),
            Self::DomainName(addr) => addr.raw_addr(),
        }
    }

    // resolve list of phone numbers associated with address (could be empty)
    pub fn resolve(&mut self, refresh: bool) -> Result<Vec<String>, String> {
        match self {
            Self::PhoneNumber(addr) => Ok(vec![addr.clone()]),
            Self::DomainName(addr) => addr.resolve(refresh),
        }
    }

    // resolve list of phone numbers associated with address and return a
    // random one
    pub fn resolve_single(&mut self, refresh: bool) -> Result<String, String> {
        let resolved = self.resolve(refresh)?;
        if resolved.is_empty() {
            return Err("unable to pick address: empty list".to_owned());
        }

        Ok(self
            .resolve(refresh)?
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_owned())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PhoneNumber(phone_addr) => phone_addr.to_owned(),
                Self::DomainName(domain_addr) => domain_addr.to_string(),
            }
        )
    }
}
