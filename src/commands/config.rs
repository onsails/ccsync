pub struct Config;

impl Config {
    pub fn execute(verbose: bool) -> anyhow::Result<()> {
        if verbose {
            println!("Executing config command");
        }

        println!("config: Not yet implemented");
        Ok(())
    }
}
