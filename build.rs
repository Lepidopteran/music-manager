use std::process::Command;

fn main() {
    if let Err(err) = build_frontend() {
        panic!("{}", err);
    };
}

fn build_frontend() -> Result<(), String> {

    let status = Command::new("bun")
        .arg("run")
        .arg("build")
        .current_dir("frontend")
        .status()
        .map_err(|err| format!("Failed to execute build command: {}", err))?;

    if !status.success() {

        let status = Command::new("bun")
            .arg("install")
            .current_dir("frontend")
            .status()
            .map_err(|err| format!("Failed to execute install command: {}", err))?;

        if !status.success() {
            return Err("Failed to install dependencies".into());
        }

        let status = Command::new("bun")
            .arg("run")
            .arg("build")
            .current_dir("frontend")
            .status()
            .map_err(|err| format!("Failed to execute build command: {}", err))?;

        if !status.success() {
            return Err("Failed to build frontend".into());
        }

    }

    Ok(())
}
