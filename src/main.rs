use clap::Parser;
use std::path::PathBuf;

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
    /// Whether to overwrite the file if it exists?
    #[clap(long, short = 'w', default_value = "false")]
    overwrite: bool,
  },
  /// Create game from game spec file.
  Import {
    /// RPC endpoint of the network node.
    #[clap(long, short, default_value = "ws://127.0.0.1:9944")]
    endpoint: String,
    /// Path to the game file from which the game configuration will be read.
    #[clap(long, short, required = true)]
    game_spec: PathBuf,
    /// Game organization account key seed.
    #[clap(long, short = 's', required = true)]
    organization_seed: String,
  },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let impex = Impex::parse();

  match impex {
    Impex::Export {
      endpoint,
      organization,
      game_spec,
      overwrite,
    } => finalbiome_impex::export_game_spec(endpoint, organization, game_spec, overwrite).await,
    Impex::Import {
      endpoint,
      game_spec,
      organization_seed,
    } => finalbiome_impex::import_game_spec(endpoint, game_spec, organization_seed).await,
  }
}
