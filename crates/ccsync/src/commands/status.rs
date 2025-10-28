use crate::cli::ConfigType;

pub struct Status;

impl Status {
    #[allow(clippy::unnecessary_wraps)]
    pub fn execute(types: &[ConfigType], verbose: bool) -> anyhow::Result<()> {
        if verbose {
            println!("Executing status command");
            println!("Types: {types:?}");
        }

        println!("status: Not yet implemented");
        Ok(())
    }
}
