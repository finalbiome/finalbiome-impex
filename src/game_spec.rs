use serde::{Deserialize, Serialize};

use crate::finalbiome::runtime_types;

type OrganizationDetails = runtime_types::pallet_organization_identity::types::OrganizationDetails<
  runtime_types::sp_runtime::bounded::bounded_vec::BoundedVec<u8>,
>;

#[derive(Serialize, Deserialize)]
/// A FinalBiome game spec struct which holds configuration of the game.
pub struct GameSpec {
  /// Version of the node
  pub version: String,
  /// Game details
  pub organization_details: OrganizationDetails,
}

pub(crate) struct GameSpecBuilder {
  /// Version of the node
  pub version: String,
  /// Game details
  pub organization_details: Option<OrganizationDetails>,
}

impl GameSpecBuilder {
  pub fn new() -> GameSpecBuilder {
    GameSpecBuilder {
      version: Default::default(),
      organization_details: Default::default(),
    }
  }

  /// Set version of the node
  pub fn version(mut self, version: String) -> GameSpecBuilder {
    self.version = version;
    self
  }

  /// Set organization details
  pub fn organization_details(
    mut self,
    organization_details: OrganizationDetails,
  ) -> GameSpecBuilder {
    self.organization_details = Some(organization_details);
    self
  }

  pub fn try_build(self) -> Result<GameSpec, Box<dyn std::error::Error>> {
    if self.organization_details.is_none() {
      return Err("Organization details not set".into());
    };

    Ok(GameSpec {
      version: self.version,
      organization_details: self.organization_details.expect("org details exists"),
    })
  }
}
