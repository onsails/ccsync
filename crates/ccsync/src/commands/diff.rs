use crate::cli::ConfigType;

pub struct Diff;

impl Diff {
    #[allow(clippy::unnecessary_wraps)]
    pub fn execute(types: &[ConfigType], verbose: bool) -> anyhow::Result<()> {
        if verbose {
            println!("Executing diff command");
            println!("Types: {types:?}");
        }

        println!("diff: Not yet implemented");
        Ok(())
    }
}
