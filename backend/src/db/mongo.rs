use mongodb::{Client, Database, options::ClientOptions};
use once_cell::sync::OnceCell;
use std::{env, error::Error};

static MONGODB_URI: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    env::var("MONGODB_URI").expect("MONGODB_URI must be set in environment")
});

static MONGODB_DATABASE: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    env::var("MONGODB_DATABASE").expect("MONGODB_DATABASE must be set in environment")
});

static MONGO_CLIENT: OnceCell<Client> = OnceCell::new();

async fn get_mongo_client() -> Result<&'static Client, Box<dyn Error>> {
    if let Some(client) = MONGO_CLIENT.get() {
        return Ok(client);
    }

    let client_options = ClientOptions::parse(MONGODB_URI.as_str()).await?;
    let client = Client::with_options(client_options)?;

    match MONGO_CLIENT.set(client) {
        Ok(_) => Ok(MONGO_CLIENT.get().unwrap()),
        Err(_) => Ok(MONGO_CLIENT.get().unwrap()),
    }
}

pub async fn init_db() -> Result<Database, Box<dyn Error>> {
    let client = get_mongo_client().await?;
    Ok(client.database(MONGODB_DATABASE.as_str()))
}
