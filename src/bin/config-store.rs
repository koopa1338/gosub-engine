use clap::{Parser, Subcommand};
use derive_more::Display;
use gosub_engine::config::{ConfigStore, StorageAdapter};
use gosub_engine::config::settings::Setting;
use gosub_engine::config::storage::json_storage::JsonStorageAdapter;
use gosub_engine::config::storage::sqlite_storage::SqliteStorageAdapter;

#[derive(Debug, Parser)]
#[clap(name = "Config-Store", version = "0.1.0", author = "Gosub")]
struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true, about = "View a setting")]
    View {
        #[clap(required = true, short = 'k', long = "key")]
        key: String,
    },
    #[clap(about = "List all settings")]
    List,
    #[clap(arg_required_else_help = true, about = "Set a setting")]
    Set {
        #[clap(required = true, short = 'k', long = "key")]
        key: String,
        #[clap(required = true, short = 'v', long = "value")]
        value: String,
    },
    #[clap(arg_required_else_help = true, about = "Search for a setting")]
    Search {
        #[clap(required = true, short = 'k', long = "key")]
        key: String,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Display)]
enum Engine {
    Sqlite,
    Json,
}

impl std::str::FromStr for Engine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sqlite" => Ok(Engine::Sqlite),
            "json" => Ok(Engine::Json),
            _ => Err(()),
        }
    }
}


#[derive(Debug, Parser)]
struct GlobalOpts {
    #[clap(short = 'e', long = "engine", global = true, default_value = "sqlite")]
    engine: Engine,
    #[clap(short = 'p', long = "path", global = true, default_value = "settings.db")]
    path: String,
}

fn main() {
    let args = Cli::parse();

    let storage_box: Box<dyn StorageAdapter>;
    match args.global_opts.engine {
        Engine::Sqlite => {
            storage_box = Box::new(SqliteStorageAdapter::new(args.global_opts.path.as_str()));
        }
        Engine::Json => {
            storage_box = Box::new(JsonStorageAdapter::new(args.global_opts.path.as_str()));
        }
    }

    let mut store = ConfigStore::new(storage_box, true);


    match args.command {
        Commands::View { key } => {
            if !store.has(key.as_str()) {
                println!("Key not found");
                return;
            }

            let info = store.get_info(key.as_str()).unwrap();
            let value = store.get(key.as_str(), None);

            println!("Key            : {}", key);
            println!("Current Value  : {}", value.to_string());
            println!("Default Value  : {}", info.default.to_string());
            println!("Description    : {}", info.description);
        }
        Commands::List => {
            for key in store.find("*".into()) {
                let value = store.get(key.as_str(), None);
                println!("{:40}: {}", key, value.to_string());
            }
        }
        Commands::Set { key, value } => {
            store.set(key.as_str(), Setting::from_string(value.as_str()).expect("incorrect value"));
        }
        Commands::Search { key } => {
            for key in store.find(key.as_str().into()) {
                let value = store.get(key.as_str(), None);
                println!("{:40}: {}", key, value.to_string());
            }
        }
    }
}
