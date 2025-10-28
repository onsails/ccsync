pub struct Config;

impl Config {
    #[allow(clippy::unnecessary_wraps)]
    pub fn execute(verbose: bool) -> anyhow::Result<()> {
        if verbose {
            println!("Executing config command");
        }

        println!("config: Not yet implemented");
        Ok(())
    }
}
