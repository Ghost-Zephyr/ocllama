use std::{env, str::FromStr};

use lazy_static::lazy_static;

use crate::{
    discord::TypeMapKey,
    ollama::Ocllama,
};

impl TypeMapKey for Ocllama {
    type Value = Ocllama;
}

fn load_env(var: &str, def: Option<String>) -> String {
    match env::var(var) {
        Ok(val) => val,
        Err(err) => if let Some(def) = def { def } else {
            panic!("Unable to load {var} from env: {err:?}")
        },
    }
}

lazy_static! {
    pub static ref TOKEN: String = load_env("DISCORD_TOKEN", None);
    pub static ref OLLAMA_URI: String = load_env("OLLAMA_URI", Some("http://localhost".to_owned()));
    pub static ref OLLAMA_PORT: u16 = {
        let port = load_env("OLLAMA_PORT", Some("11434".to_owned()));
        match u16::from_str(&port) {
            Err(err) => panic!("Unable to parse number from OLLAMA_PORT={port}: {err}"),
            Ok(port) => port,
        }
    };
    pub static ref OWNER_ID: u64 = {
        let owner_id = load_env("OWNER_ID", Some("231931783262765057".to_owned()));
        match u64::from_str(&owner_id) {
            Err(err) => panic!("Unable to parse number from OWNER_ID={owner_id}: {err}"),
            Ok(owner_id) => owner_id,
        }
    };
}
