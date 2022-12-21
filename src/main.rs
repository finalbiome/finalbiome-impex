use clap::Parser;
use std::{
	path::{Path, PathBuf}
};

#[derive(Parser)]
#[clap(
  name = "finalbiome-impex",
  author = "FinalBiome Devs <https://github.com/finalbiome>",
  about = "A utility to easily create a game spec for testnet.",
  version
)]
enum Impex {
  /// Export game spec to the file.
  Export {
    /// RPC endpoint of the network node.
    #[clap(long, short, default_value = "ws://127.0.0.1:9944")]
    endpoint: String,
    /// Game address in SS58 format.
    #[clap(long, short, required = true)]
    organization: String,
    /// Path to the game file to which the game configuration will be written.
		#[clap(long, short, default_value = "./game_spec.json")]
    game_spec: PathBuf,
  },
  /// Create game from game spec file.
  Import {
    /// RPC endpoint of the network node.
    #[clap(long, short, default_value = "ws://127.0.0.1:9944")]
    endpoint: String,
    /// Game address in SS58 format.
    #[clap(long, short, required = true)]
    organization: String,
    /// Path to the game file from which the game configuration will be read.
    #[clap(long, short, required = true)]
    game_spec: PathBuf,
    /// Game organization account key seed.
    #[clap(long, short ='s', required = true)]
    organization_seed: String,
  },
}

impl Impex {
  /// Returns the path where the game spec should be saved or read from.
  fn game_spec_path(&self) -> &Path {
    match self {
      Impex::Export { game_spec, .. } => game_spec.as_path(),
      Impex::Import { game_spec, .. } => game_spec.as_path(),
    }
  }
}

fn main() -> Result<(), String> {
  
  let impex = Impex::parse();
  let game_spec_path = impex.game_spec_path().to_path_buf();

  match impex {
      Impex::Export { endpoint, organization, game_spec } => println!("Export"),
      Impex::Import { endpoint, organization, game_spec, organization_seed } => println!("Import"),
  }
  
  Ok(())
}
