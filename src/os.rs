use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub struct AppRegistry {
    registry: HashMap<String, Rc<dyn App>>,
}

impl AppRegistry {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        let modbaynet = Rc::new(ModbayNet::new());

        for addr in modbaynet.register() {
            // NOTE: in production collisions and hierarchy should be checked
            // carefully
            registry.insert(addr, Rc::clone(&modbaynet) as Rc<dyn App>);
        }

        Self { registry }
    }

    // try to get general website info from app
    pub fn get_general_info(&self, domain: &str) -> Option<String> {
        self.registry.get(domain).map(|app| app.general_info())
    }

    // try to get extra info about subaddress from app
    pub fn get_extra_info(&self, domain: &str, path: &str) -> Option<String> {
        if let Some(app) = self.registry.get(domain) {
            app.fetch_info(path)
        } else {
            None
        }
    }
}

pub trait App: fmt::Debug {
    // list of domains app is serving
    fn register(&self) -> Vec<String>;

    // get general unconditional info
    fn general_info(&self) -> String;

    // get info from specific path
    fn fetch_info(&self, path: &str) -> Option<String>;
}

// theoretical classified advertisements app like Avito or Ozon
#[derive(Debug)]
struct ModbayNet {
    // external database accessible via API call
    order_db: HashMap<String, ModbayOrder>,
}

#[derive(Debug)]
struct ModbayUser {
    username: String,
    reputation: i32,
}

impl fmt::Display for ModbayUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "|\tUsername: {}\n|\tReputation: {}",
            self.username, self.reputation,
        )
    }
}

#[derive(Debug)]
struct ModbayOrder {
    user: ModbayUser,
    name: String,
    photo_url: String,
}

impl fmt::Display for ModbayOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\n|Order: {}\n|Image (rendered): {}\n|User: \n{}",
            self.name, self.photo_url, self.user
        )
    }
}

impl ModbayNet {
    fn new() -> Self {
        let mut order_db = HashMap::new();

        order_db.insert(
            "12345".to_owned(),
            ModbayOrder {
                user: ModbayUser {
                    username: "untrusted_user".to_owned(),
                    reputation: -8,
                },
                name: "sandals".to_owned(),
                photo_url: "https://modbay.net/static/sandals.jpg".to_owned(),
            },
        );

        Self { order_db }
    }

    fn fetch_order(&self, order_id: &str) -> Option<&ModbayOrder> {
        // API call happens here.
        // Since app handles this internally via callback, it can use access
        // tokens if user is logged in.
        self.order_db.get(order_id)
    }
}

impl App for ModbayNet {
    fn register(&self) -> Vec<String> {
        vec!["modbay.net".to_owned()]
    }

    fn general_info(&self) -> String {
        "Classified advertisements website".to_owned()
    }

    fn fetch_info(&self, path: &str) -> Option<String> {
        if let Some((id, tp)) = path.split_once('.') {
            match tp {
                "order" => self.fetch_order(id).map(|o| o.to_string()),
                _ => None,
            }
        } else {
            None
        }
    }
}
