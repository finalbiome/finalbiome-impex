use sp_core::storage::StorageKey;
use subxt::OnlineClient;

use crate::{ResultOf};

/// Iterates over all keys in a map by prefix key
pub(crate) struct AllKeyIter<'a, T>
where
  T: subxt::Config,
{
  api: &'a OnlineClient<T>,
  query_key: Vec<u8>,
  page_size: u32,
  block_hash: T::Hash,
  start_key: Option<StorageKey>,
  buffer: Vec<StorageKey>,
}

impl<'a, T> AllKeyIter<'a, T>
where
  T: subxt::Config,
{
  /// Create new iterator
  pub fn new(api: &OnlineClient<T>, query_key: Vec<u8>, block_hash: T::Hash, page_size: u32) -> AllKeyIter<T> {
    AllKeyIter {
      api,
      query_key,
      page_size,
      block_hash,
      start_key: None,
      buffer: Default::default(),
    }
  }
  /// Returns the next key from a storage.
  pub async fn next(&mut self) -> ResultOf<Option<StorageKey>>
  {
    loop {
      if let Some(k) = self.buffer.pop() {
        return Ok(Some(k))
      } else {
        let start_key = self.start_key.take();
        let mut keys = self
          .api
          .storage()
          .fetch_keys(&self.query_key, self.page_size, start_key.as_ref().map(|k| &*k.0), Some(self.block_hash))
          .await?;
        
        if keys.is_empty() {
          return Ok(None)
        }
        
        self.start_key = keys.last().cloned();
        self.buffer.append(&mut keys);
      }
    }
  }
}
