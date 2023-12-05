mod backuper;
mod logger;
mod exts;

#[cfg(not(target_os = "windows"))]
const DIRECTORY: &'static str = "/MongoBackups";

#[cfg(target_os = "windows")]
const DIRECTORY: &'static str = "C:\\MongoBackups";



#[cfg(not(target_os = "windows"))]
fn main() {
    loop {
        println!("{} Write command... // Write \"help\" to get commands", logger::colors::green("{INPUT}"));

        let readed = exts::read_line();
        match readed.as_str() {
            "help" => {
                logger::info("Command list:");
                logger::info("| help - Get a list of commands");
                logger::info("| run - Run the backup script");
                logger::info("| quit - Close the app");
            }
            "run" => {
                backuper::run();
            }
            "quit" => {
                exts::close_proc();
            }
            _ => {
                logger::warn_string(format!("Unknow command: {readed}"));
            }
        }
    }
}



#[cfg(target_os = "windows")]
const SERVICENAME: &'static str = "mongo_backuper";

#[cfg(target_os = "windows")]
#[macro_use]
extern crate windows_service;

#[cfg(target_os = "windows")]
use {
    tokio::time::Duration,
    std::{path::Path, fs, process, ffi::OsString},
    windows_service::service_dispatcher,
    windows_service::service_control_handler::{self, ServiceControlHandlerResult},
    windows_service::service_manager::{ServiceManager, ServiceManagerAccess},
    windows_service::service::{
        ServiceControl, ServiceInfo, ServiceType, 
        ServiceStartType, ServiceAccess, ServiceErrorControl, 
        ServiceState, ServiceStatus, ServiceControlAccept, ServiceExitCode
    }
};

#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, service_init);

#[cfg(target_os = "windows")]
fn service_init(arguments: Vec<OsString>) {
    if let Err(err) = service_run(arguments) {
        logger::error_string(format!("Service Init: {err}"));
    }
}

#[cfg(target_os = "windows")]
fn service_run(_: Vec<OsString>) -> Result<(), windows_service::Error> {

    let event_handler = move | control_event | -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                process::exit(0x0000);
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICENAME, event_handler)?;

    let proccess_id: Option<u32> = Some(process::id());
    let service_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: proccess_id,
    };
    
    status_handle.set_service_status(service_status)?;

    backuper::run();
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), windows_service::Error> {
    let mut run_dir = format!("{}", std::env::var("USERPROFILE").unwrap_or_default());
    run_dir.remove(0);

    if run_dir.starts_with(":\\WINDOWS\\system32") {
        service_dispatcher::start(SERVICENAME, ffi_service_main)?;
    } else {
        loop {
            println!("{} Write command... // Write \"help\" to get commands", logger::colors::green("{INPUT}"));

            let readed = exts::read_line();
            match readed.as_str() {
                "help" => {
                    logger::info("Command list:");
                    logger::info("| help - Get a list of commands");
                    logger::info("| install - Install a service for automatic backups");
                    logger::info("| uninstall - Remove a service for automatic backups");
                    logger::info("| restart - Restart a service for automatic backups");
                    logger::info("| run - Run the backup script");
                    logger::info("| quit - Close the app");
                }
                "install" => {
                    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
                    let service_manager = match ServiceManager::local_computer(None::<&str>, manager_access) {
                        Ok(res) => res,
                        Err(err) => {
                            logger::warn_string(format!("Failed to create a ServiceManager session: {err}"));
                            continue;
                        }
                    };

                    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
                    if let Ok(service) = service_manager.open_service(SERVICENAME, service_access) {
                        if let Err(err) = service.delete() {
                            logger::warn_string(format!("Failed to delete old service: {err}"));
                        }
                        match service.query_status() {
                            Ok(status) => {
                                if status.current_state != ServiceState::Stopped {
                                    if let Err(err) = service.stop() {
                                        logger::warn_string(format!("Failed to stop old service: {err}"));
                                    }
                                }
                            },
                            Err(err) => {
                                logger::warn_string(format!("Failed to get current status of old service: {err}"));
                            }
                        }
                    }

                    
                    let service_file_path = Path::new("C:\\ProgramData\\MongoBackuper");

                    if service_file_path.exists() {
                        if let Err(err) = fs::remove_dir_all(service_file_path) {
                            logger::warn_string(format!("Error when deleting a exists directory: {err}"));
                        }
                    }

                    if let Err(err) = fs::create_dir_all(service_file_path) {
                        logger::warn_string(format!("Error when creating a directory: {err}"));
                        continue;
                    }

                    let current_path = match std::env::current_exe() {
                        Ok(path) => path,
                        Err(err) => {
                            logger::warn_string(format!("Error when getting the location of the current file: {err}"));
                            continue;
                        }
                    };
                    
                    let exec_file_path = service_file_path.join("MongoBackuper.exe");
                    if let Err(err) = fs::copy(current_path, &exec_file_path) {
                        logger::warn_string(format!("Error when copying a file: {err}"));
                        continue;
                    }

                    
                    let service_info = ServiceInfo {
                        name: OsString::from(SERVICENAME),
                        display_name: OsString::from("MongoBackuper"),
                        service_type: ServiceType::OWN_PROCESS,
                        start_type: ServiceStartType::AutoStart,
                        error_control: ServiceErrorControl::Normal,
                        executable_path: exec_file_path,
                        launch_arguments: vec![],
                        dependencies: vec![],
                        account_name: None,
                        account_password: None,
                    };

                    let service_open_access = ServiceAccess::CHANGE_CONFIG | ServiceAccess::START;
                    let service = match service_manager.create_service(&service_info, service_open_access) {
                        Ok(res) => res,
                        Err(err) => {
                            logger::warn_string(format!("Failed to create service: {err}"));
                            continue;
                        }
                    };

                    let args: [OsString; 0] = [];
                    if let Err(err) = service.start(&args) {
                        logger::warn_string(format!("Failed to start service: {err}"));
                    }

                    if let Err(err) = service.set_description("Create backups of MongoDB") {
                        logger::warn_string(format!("Failed to change service desc: {err}"));
                    }

                    logger::info("Service created");
                }
                "uninstall" => {
                    let manager_access = ServiceManagerAccess::CONNECT;
                    let service_manager = match ServiceManager::local_computer(None::<&str>, manager_access) {
                        Ok(res) => res,
                        Err(err) => {
                            logger::warn_string(format!("Failed to create a ServiceManager session: {err}"));
                            continue;
                        }
                    };

                    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
                    if let Ok(service) = service_manager.open_service(SERVICENAME, service_access) {
                        if let Err(err) = service.delete() {
                            logger::warn_string(format!("Failed to delete service: {err}"));
                        }
                        match service.query_status() {
                            Ok(status) => {
                                if status.current_state != ServiceState::Stopped {
                                    if let Err(err) = service.stop() {
                                        logger::warn_string(format!("Failed to stop service: {err}"));
                                    }
                                }
                            },
                            Err(err) => {
                                logger::warn_string(format!("Failed to get current status of service: {err}"));
                            }
                        }
                    }

                    let service_file_path = Path::new("C:\\ProgramData\\MongoBackuper");
                    
                    if service_file_path.exists() {
                        if let Err(err) = fs::remove_dir_all(service_file_path) {
                            logger::warn_string(format!("Error when deleting a exists directory: {err}"));
                        }
                    }

                    logger::info("Service deleted");
                }
                "restart" => {
                    let manager_access = ServiceManagerAccess::CONNECT;
                    let service_manager = match ServiceManager::local_computer(None::<&str>, manager_access) {
                        Ok(res) => res,
                        Err(err) => {
                            logger::warn_string(format!("Failed to create a ServiceManager session: {err}"));
                            continue;
                        }
                    };

                    let service_access = ServiceAccess::STOP | ServiceAccess::START;
                    if let Ok(service) = service_manager.open_service(SERVICENAME, service_access) {
                        if let Err(err) = service.stop() {
                            logger::warn_string(format!("Failed to stop service: {err}"));
                        }

                        let args: [OsString; 0] = [];
                        if let Err(err) = service.start(&args) {
                            logger::warn_string(format!("Failed to start service: {err}"));
                        }
                    }

                    logger::info("Service restarted");
                }
                "run" => {
                    backuper::run();
                }
                "quit" => {
                    exts::close_proc();
                }
                _ => {
                    logger::warn_string(format!("Unknow command: {readed}"));
                }
            }
        }
    }

    return Ok(());
}