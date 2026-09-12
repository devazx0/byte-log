use crate::cli::args::ServiceAction;
use crate::error::Result;
use crate::service::ServiceManager;

pub fn handle_service(action: ServiceAction) -> Result<()> {
    match action {
        ServiceAction::Install => {
            ServiceManager::install()?;
            println!("Service registered successfully.");
        }
        ServiceAction::Start => {
            ServiceManager::start()?;
            println!("Service started.");
        }
        ServiceAction::Stop => {
            ServiceManager::stop()?;
            println!("Service stopped.");
        }
        ServiceAction::Restart => {
            let _ = ServiceManager::stop();
            ServiceManager::start()?;
            println!("Service restarted.");
        }
        ServiceAction::Status => {
            let status = ServiceManager::status()?;
            println!("{}", status);
        }
        ServiceAction::Uninstall => {
            ServiceManager::uninstall()?;
            println!("Service unregistered.");
        }
    }
    Ok(())
}
