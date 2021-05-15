mod address;
mod os;
mod session;

use std::env;
use std::process;

use crate::address::Address;
use crate::os::AppRegistry;
use crate::session::Session;

// outgoing call processing
fn outgoing_call(addr: &mut Address) -> Result<Session, String> {
    eprintln!("ocall:  calling: {}", &addr);
    let ph_number = addr.resolve_single(false)?;
    eprintln!("ocall: resolved: {}", &addr);

    let session = Session::new(&ph_number);

    eprintln!("ocall:  session: {}", &session);
    eprintln!();

    Ok(session)
}

// incoming call processing
fn incoming_call(
    ph_number: &str,
    addr: &mut Address,
    app_registry: &AppRegistry,
) -> Result<Session, String> {
    eprintln!(
        "icall    call from: {}, addr: {}",
        ph_number,
        addr.to_string(),
    );

    match addr {
        Address::PhoneNumber(..) => {
            eprintln!("icall unresolvable: no domain name provided, nothing to resolve");
        }
        Address::DomainName(dn) => {
            let resolved = dn.resolve(false)?;

            if !resolved.contains(&ph_number.to_owned()) {
                return Err("resolved numbers do not include incoming number".to_owned());
            }

            if let Some(general_info) = dn.general_info(app_registry) {
                eprintln!("icall  description: {}", general_info);
            }

            if let Some(extra_info) = dn.fetch_extra_info(app_registry) {
                eprintln!("icall        extra: {}", extra_info);
            }
        }
    }

    let session = Session::new(ph_number);

    eprintln!("icall      session: {}", &session);
    eprintln!();

    Ok(session)
}

fn main_() -> Result<(), String> {
    let argv: Vec<String> = env::args().collect();

    let argc = argv.len();
    if argc == 1 {
        return default();
    } else if argc == 3 {
        let mut addr = Address::new(&argv[2])?;

        match &argv[1][..] {
            "ocall" => outgoing_call(&mut addr)?,
            "icall" => {
                let app_registry = AppRegistry::new();

                let ph_number = addr.resolve_single(false)?;
                incoming_call(&ph_number, &mut addr, &app_registry)?
            }
            _ => {
                eprintln!("unknown command: {}. expected [ocall, icall]", argv[1]);
                process::exit(1);
            }
        };
    } else {
        eprintln!("Usage: ph-record [[ocall|icall] <address>]");
        process::exit(1);
    }

    Ok(())
}

fn default() -> Result<(), String> {
    let mut addr_num = Address::new("+1-212-555-0101")?;
    let mut addr_resolvable = Address::new("modbay.net")?;
    let mut addr_resolvable_extra_info = Address::new("12345.order._tel.modbay.net")?;
    let mut addr_unresolvable = Address::new("google.com")?;

    eprintln!("--- OUTGOING CALLS ---");
    outgoing_call(&mut addr_num)?;
    outgoing_call(&mut addr_resolvable)?;
    outgoing_call(&mut addr_resolvable_extra_info)?;

    eprintln!(
        "calling {} (unresolvable): {:?}\n",
        addr_unresolvable.clone(),
        addr_unresolvable.resolve(false)?
    );

    let app_registry = AppRegistry::new();

    eprintln!("--- INCOMING CALLS ---");
    incoming_call(&addr_num.raw_addr(), &mut addr_num, &app_registry)?;
    incoming_call(&addr_num.raw_addr(), &mut addr_resolvable, &app_registry)?;
    incoming_call(
        &addr_num.raw_addr(),
        &mut addr_resolvable_extra_info,
        &app_registry,
    )?;
    incoming_call(&addr_num.raw_addr(), &mut addr_unresolvable, &app_registry)?;

    Ok(())
}

fn main() {
    if let Err(e) = main_() {
        eprint!("error: {}", e);
    }
}
