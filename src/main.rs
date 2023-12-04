use futures_util::{StreamExt, stream::FuturesUnordered};
use bson::RawDocumentBuf;
use serde::{Deserialize, Serialize};
use mongodb::Client;
use time::{OffsetDateTime, Date, Time, Month};
use std::{path::Path, fs::{self, File}, io::Write, process};
use tokio::time::{sleep, Duration};

mod logger;
mod exts;

#[cfg(not(target_os = "windows"))]
const DIRECTORY: &'static str = "/MongoBackups";

#[cfg(target_os = "windows")]
const DIRECTORY: &'static str = "C:\\MongoBackups";

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigConnect {
    pub name: String,
    pub url: String,
    pub interval: f64,
    #[serde(rename = "removeOld")]
    pub remove_old: f64,
}

fn main() {
    logger::info("MongoDB Backuper started");

    if !Path::new(&DIRECTORY).exists() {
        logger::debug("MongoDB directory not found. Creating...");

        let result = fs::create_dir_all(&DIRECTORY);

        if result.is_err() {
            logger::error_string(format!("Failed to create directory for MongoDB Backups at {}", &DIRECTORY));
            close_proc();
            return;
        }
    }

    let config_path = Path::new(&DIRECTORY).join("config.js");

    if !config_path.exists() {
        logger::debug("Config file not found. Creating...");

        let result = fs::write(&config_path, get_config_example());

        if result.is_err() {
            logger::error("Failed to create config file");
            close_proc();
            return;
        }
    }

    let config_pre_data = fs::read_to_string(&config_path);
    if config_pre_data.is_err() {
        logger::error("Failed to read config file");
        close_proc();
        return;
    }

    let config_data = normalize_config_file(config_pre_data.unwrap_or_default());
    let pre_cfg = serde_json::from_str(&config_data);
    let config: Vec<ConfigConnect> = match pre_cfg {
        Ok(res) => res,
        Err(err) => {
            logger::error_string(err.to_string());
            close_proc();
            return;
        },
    };

    logger::debug_string(format!("Collections count: {}", config.len()));

    if config.len() == 0 {
        logger::error("Config doesn't have MongoDB connections");
        close_proc();
        return;
    }

    let mut procs = Vec::new();
    for cfg_connect in config {
        let task = async move {
            loop {
                backup(&cfg_connect).await;
                sleep(Duration::from_secs((&cfg_connect.interval * 3600 as f64) as u64)).await;
            }
        };
        let task = Box::pin(task);
        procs.push(task);
    }


    let rt = match tokio::runtime::Runtime::new() {
        Ok(res) => res,
        Err(err) => {
            logger::error_string(format!("Failed to create tokio runtime: {err}"));
            close_proc();
            return;
        }
    };

    rt.block_on(async {
        let futures = FuturesUnordered::from_iter(procs);
        futures.collect::<()>().await;
    });

    logger::warn("All processes of backup have been stopped");

    close_proc();
}

fn close_proc() {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(res) => res,
        Err(err) => {
            logger::error_string(format!("Failed to create tokio runtime in close_proc(): {err}"));
            return;
        }
    };

    rt.block_on(async {
        logger::warn("Window will be closed after 5 seconds");
        sleep(Duration::from_secs(5)).await;
    });

    process::exit(0x0100);
}


async fn backup(config: &ConfigConnect) {
    if config.interval < 0.05 {
        logger::warn_string(format!("Interval can not be lower than 0.05 [3 min] ({}) of \"${}\"", &config.interval, &config.name));
        return;
    }

    let root_dir_path = Path::new(&DIRECTORY).join("Backups").join(&config.name);

    delete_old_dirs(&root_dir_path, &config);

    logger::info_string(format!("Backing up the collection \"{}\" has been started", &config.name));

    let dir_path = Path::new(&root_dir_path).join(exts::get_date_file());
    
    if dir_path.exists() {
        fs::remove_dir_all(&dir_path).unwrap_or_default();
    }

    match fs::create_dir_all(&dir_path) {
        Ok(_) => {},
        Err(err) => {
            logger::warn_string(
                format!("Failed to create directory: {} > {}",
                dir_path.to_str().unwrap_or_default(), err.to_string())
            );
            return;
        }
    }

    let pre_client = Client::with_uri_str(&config.url).await;
    let client = match pre_client {
        Ok(res) => res,
        Err(err) => {
            logger::error_string(err.to_string());
            return;
        }
    };

    let pre_databases = client.list_database_names(None, None).await;
    let databases = match pre_databases {
        Ok(res) => res,
        Err(err) => {
            logger::error_string(err.to_string());
            return;
        }
    };

    for db_name in databases {
        if db_name == "config" || db_name == "local" {
            continue;
        }
        
        let db = client.database(&db_name);

        let pre_collections = db.list_collection_names(None).await;
        let collections = match pre_collections {
            Ok(res) => res,
            Err(err) => {
                logger::error_string(err.to_string());
                continue;
            }
        };

        let db_dir_path = Path::new(&dir_path).join(&db_name);
        
        if db_dir_path.exists() {
            fs::remove_dir_all(&db_dir_path).unwrap_or_default();
        }

        match fs::create_dir_all(&db_dir_path) {
            Ok(_) => {},
            Err(err) => {
                logger::warn_string(
                    format!("Failed to create directory: {} > {}",
                    db_dir_path.to_str().unwrap_or_default(), err.to_string())
                );
                continue;
            }
        }

        logger::debug_string(format!("Creating Backup of \"{db_name}\" in \"{}\"", &config.name));

        for collection_name in collections {
            let collection = db.collection::<RawDocumentBuf>(&collection_name);

            let cursor = match collection.find(None, None).await {
                Ok(cursor) => cursor,
                Err(_) => continue,
            };

            let pre_file = File::create(Path::new(&db_dir_path).join(format!("{collection_name}.bson")));
            let mut file = match pre_file {
                Ok(f) => f,
                Err(err) => {
                    format!("Failed to create file for collection \"{}\" > {}",
                    collection_name, err.to_string());
                    continue;
                }
            };
            
            let docs = cursor.collect::<Vec<_>>().await;
            for pre_doc in &docs {
                match pre_doc {
                    Ok(doc) => {
                        _ = file.write_all(doc.as_bytes()).unwrap_or_default();
                    },
                    Err(_) => {}
                }
            }
        }
    };

    logger::info_string(format!("Backup of the collection \"{}\" completed", &config.name));
}


fn get_config_example() -> &'static str {
r#"[
    {
        "name": "mydb", // The name of the database (for the backup directory), can be arbitrary
        "url": "mongodb://localhost", // Link-connect to MongoDB
        "interval": 4, // Sets the interval for database backup (in hours)
        "removeOld": 30 // Automatically deletes old backups that exceed the specified number of backups (In days) *But keeps one backup in any occasions.
    },
    { // To backup multiple databases
        "name": "mydb2",
        "url": "mongodb://user:password@host:port",
        "interval": 12,
        "removeOld": 15
    }
]"#
}

fn normalize_config_file(content: String) -> String {
    let mut res = String::new();
    let arr = content.split('\n');

    for cont in arr {
        let parse_cont: Vec<_> = cont.split(" // ").collect();
        if let Some(first) = parse_cont.first() {
            //res.push_str(first.trim());
            res.push_str(format!("{first}\n").as_str());
        }
    }

    return res;
}

fn delete_old_dirs(root_dir_path: &Path, config: &ConfigConnect) {
    if root_dir_path.exists() {
        logger::debug_string(format!("Checking and deleting old backups of \"{}\"", &config.name));

        let mut files_vec: Vec<String> = Vec::new();

        match fs::read_dir(&root_dir_path) {
            Ok(files) => {
                for file in files {
                    match file {
                        Ok(file_name) => {
                            let name = file_name.file_name().into_string().unwrap_or_default();

                            match fs::remove_dir(Path::new(&root_dir_path).join(&name)) {
                                Ok(_) => continue,
                                Err(_) => {}
                            }

                            if !name.contains(".") || !name.contains(" ") || !name.contains("-") {
                                continue;
                            }

                            files_vec.push(name);
                        },
                        Err(err) => {
                            logger::debug_string(err.to_string());
                        }
                    }
                }
            },
            Err(err) => {
                logger::debug_string(err.to_string());
            }
        }

        for name in files_vec {

            match fs::read_dir(&root_dir_path) {
                Ok(files) => {
                    if files.count() < 2 {
                        return;
                    }
                },
                Err(_) => {}
            }

            let arr1: Vec<_> = name.split(' ').collect();
            
            let year: i32;
            let month: Month;
            let day: u8; 
            let hours: u8;
            let minutes: u8;

            if let Some(first) = arr1.first() {
                let local_arr: Vec<_> = first.trim().split('.').collect();

                if let Some(hrs) = local_arr.first() {
                    day = hrs.to_string().parse::<u8>().unwrap_or_default();
                } else {
                    continue;
                }

                if local_arr.len() > 1 {
                    let pre_mnth = local_arr[1];
                    let pre_mnth = pre_mnth.to_string().parse::<u8>().unwrap_or_default();
                    month = match pre_mnth {
                        1 => Month::January,
                        2 => Month::February,
                        3 => Month::March,
                        4 => Month::April,
                        5 => Month::May,
                        6 => Month::June,
                        7 => Month::July,
                        8 => Month::August,
                        9 => Month::September,
                        10 => Month::October,
                        11 => Month::November,
                        12 => Month::December,
                        _ => Month::January,
                    }
                } else {
                    continue;
                }

                if let Some(min) = local_arr.last() {
                    year = min.to_string().parse::<i32>().unwrap_or_default();
                } else {
                    continue;
                }
            } else {
                continue;
            }
            
            if let Some(last) = arr1.last() {
                let local_arr: Vec<_> = last.trim().split('-').collect();

                if let Some(hrs) = local_arr.first() {
                    hours = hrs.to_string().parse::<u8>().unwrap_or_default();
                } else {
                    continue;
                }

                if let Some(min) = local_arr.last() {
                    minutes = min.to_string().parse::<u8>().unwrap_or_default();
                } else {
                    continue;
                }
            } else {
                continue;
            }
            
            let date = match Date::from_calendar_date(year, month, day) {
                Ok(res) => res,
                Err(_) => continue
            };
            let time = match Time::from_hms(hours, minutes, 00) {
                Ok(res) => res,
                Err(_) => continue
            };

            let mut datetime = OffsetDateTime::now_utc();
            datetime = datetime.replace_date(date);
            datetime = datetime.replace_time(time);

            let unix = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());

            let total = unix.unix_timestamp() - datetime.unix_timestamp();
            if total > (config.remove_old * 86400 as f64) as i64 { // 60 * 60 * 24
                logger::debug_string(format!("Removing directory \"{name}\" of \"{}\"", &config.name));
                fs::remove_dir_all(Path::new(&root_dir_path).join(&name)).unwrap_or_default();
            }
        }
    }
}