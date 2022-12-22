use serde::{Deserialize, Serialize};

use crate::{finalbiome::runtime_types, FungibleAssetIds};

type OrganizationDetails = runtime_types::pallet_organization_identity::types::OrganizationDetails<
  runtime_types::sp_runtime::bounded::bounded_vec::BoundedVec<u8>,
>;
type OrganizationMembers = Vec<sp_runtime::AccountId32>;

#[derive(Serialize, Deserialize)]
/// A FinalBiome game spec struct which holds configuration of the game.
pub struct GameSpec {
  /// Version of the node
  pub version: String,
  /// Hash of the network block when the game spec was exported
  pub hash: String,
  /// Game details
  pub organization_details: OrganizationDetails,
  /// Members of the organization
  pub organization_members: OrganizationMembers,
  /// Fungible assets
  pub fa: FungibleAssetIds,
}

#[derive(Default)]
pub(crate) struct GameSpecBuilder {
  /// Version of the node
  pub version: String,
  /// Hash of the network block when the game spec was exported
  pub hash: String,
  /// Game details
  pub organization_details: Option<OrganizationDetails>,
  /// Members of the organization
  pub organization_members: Option<OrganizationMembers>,
  /// Fungible assets
  pub fa: Option<FungibleAssetIds>,
}

impl GameSpecBuilder {
  pub fn new() -> GameSpecBuilder {
    GameSpecBuilder::default()
  }

  /// Set version of the node
  pub fn version(mut self, version: String) -> GameSpecBuilder {
    self.version = version;
    self
  }
  /// Set hash of the state
  pub fn hash(mut self, hash: String) -> GameSpecBuilder {
    self.hash = hash;
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

  /// Set members of the organization
  pub fn organization_members(
    mut self,
    organization_members: OrganizationMembers,
  ) -> GameSpecBuilder {
    self.organization_members = Some(organization_members);
    self
  }

  /// Set FA of the organization
  pub fn fa(mut self, fa: FungibleAssetIds) -> GameSpecBuilder {
    self.fa = Some(fa);
    self
  }

  pub fn try_build(self) -> Result<GameSpec, Box<dyn std::error::Error>> {
    if self.organization_details.is_none() {
      return Err("Organization details not set".into());
    };

    Ok(GameSpec {
      version: self.version,
      hash: self.hash,
      organization_details: self.organization_details.expect("org details exists"),
      organization_members: self.organization_members.expect("org members exists"),
      fa: self.fa.expect("fa exists"),
    })
  }
}
