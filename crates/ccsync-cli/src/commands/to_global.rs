use crate::cli::{ConfigType, ConflictMode};

pub struct ToGlobal;

impl ToGlobal {
    #[allow(clippy::unnecessary_wraps)]
    pub fn execute(types: &[ConfigType], conflict: &ConflictMode, verbose: bool, dry_run: bool) -> anyhow::Result<()> {
        if verbose {
            println!("Executing to-global command");
            println!("Types: {types:?}");
            println!("Conflict mode: {conflict:?}");
            println!("Dry run: {dry_run}");
        }

        println!("to-global: Not yet implemented");
        Ok(())
    }
}
