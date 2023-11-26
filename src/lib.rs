use bevy::{
    prelude::{AssetServer, Handle, Resource},
    utils::hashbrown::HashMap,
};
use std::{hash::Hash, sync::RwLock};

// Creates an AssetManager<$key_kind, $asset_kind> with unloaded assets
#[macro_export]
macro_rules! lazy_asset_manager {
    (<$key_kind:ty, $asset_kind:ty> binds $asset_server:expr) => {
        $crate::AssetManager::<$key_kind, $asset_kind>::new($asset_server)
    };

    (<$key_kind:ty, $asset_kind:ty> binds $asset_server:expr, { $($key:expr => $path:expr),* $(,)? }) => ({
        let asset_manager = $crate::AssetManager::<$key_kind, $asset_kind>::new($asset_server);
        asset_manager.insert_many(&vec![$(($key, $path)),*]);

        asset_manager
    });
}

// Creates an AssetManager<$key_kind, $asset_kind> with loaded assets
#[macro_export]
macro_rules! loaded_asset_manager {
    (<$key_kind:ty, $asset_kind:ty> binds $asset_server:expr) => {
        $crate::AssetManager::<$key_kind, $asset_kind>::new($asset_server)
    };

    (<$key_kind:ty, $asset_kind:ty> binds $asset_server:expr, { $($key:expr => $path:expr),* $(,)? }) => ({
        let asset_manager = $crate::AssetManager::<$key_kind, $asset_kind>::new($asset_server);
        asset_manager.insert_many_loaded(&vec![$(($key, $path)),*]);

        asset_manager
    });
}

// Creates an AssetManager<$key_kind, $asset_kind> with a combination of loaded and unloaded assets
#[macro_export]
macro_rules! mixed_asset_manager {
    (<$key_kind:ty, $asset_kind:ty> binds $asset_server:expr) => {
        $crate::AssetManager::<$key_kind, $asset_kind>::new($asset_server)
    };

    (<$key_kind:ty, $asset_kind:ty> binds $asset_server:expr, { $($load_kind:expr, $key:expr => $path:expr),* $(,)? }) => ({
        let asset_manager = $crate::AssetManager::<$key_kind, $asset_kind>::new($asset_server);
        let mut lazy = vec![];
        let mut loaded = vec![];

        $(match $load_kind {
            $crate::LoadStyle::Lazy => lazy.insert(($key, $path)),
            $crate::LoadStyle::Loaded => loaded.insert(($key, $path)),
        })*

        asset_manager.insert_many(&lazy);
        asset_manager.insert_many(&loaded);

        asset_manager
    });
}

// Represents the load style for an asset in mixed_asset_manager!
pub enum LoadStyle {
    Lazy,
    Loaded,
}

enum AssetHandle<Asset>
where
    Asset: bevy::asset::Asset,
{
    Lazy(String),
    Loaded(Handle<Asset>),
}

#[derive(Resource)]
pub struct AssetManager<Key, Asset>
where
    Key: PartialEq + Eq + Hash,
    Asset: bevy::asset::Asset,
{
    assets: RwLock<HashMap<Key, AssetHandle<Asset>>>,
    asset_server: AssetServer,
}

// TODO: add bulk write methods and use those in the macros
impl<Key, Asset> AssetManager<Key, Asset>
where
    Key: PartialEq + Eq + Hash + Copy,
    Asset: bevy::asset::Asset,
{
    pub fn new(asset_server: AssetServer) -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
            asset_server,
        }
    }

    pub fn insert(&self, key: Key, path: &str) {
        self.assets
            .write()
            .unwrap()
            .insert(key, AssetHandle::Lazy(path.to_owned()));
    }

    pub fn insert_many(&self, pairs: &[(Key, &str)]) {
        let mut lock = self.assets.write().unwrap();

        pairs.into_iter().for_each(|(key, path)| {
            lock.insert(*key, AssetHandle::Lazy(path.to_owned().to_owned()));
        });
    }

    pub fn insert_loaded(&self, key: Key, path: &str) {
        self.assets.write().unwrap().insert(
            key,
            AssetHandle::Loaded(self.asset_server.load(path.to_owned())),
        );
    }

    pub fn insert_many_loaded(&self, pairs: &[(Key, &str)]) {
        let mut lock = self.assets.write().unwrap();

        pairs.into_iter().for_each(|(key, path)| {
            lock.insert(
                *key,
                AssetHandle::Loaded(self.asset_server.load(path.to_owned().to_owned())),
            );
        });
    }

    pub fn load(&self, key: Key) {
        if let Some(asset) = self.assets.write().unwrap().get_mut(&key) {
            match asset {
                AssetHandle::Lazy(path) => {
                    *asset = AssetHandle::Loaded(self.asset_server.load(path.to_owned()))
                }
                AssetHandle::Loaded(_) => {}
            }
        }
    }

    pub fn load_many(&self, keys: &[Key]) {
        let mut lock = self.assets.write().unwrap();

        keys.into_iter().for_each(|key| {
            if let Some(asset) = lock.get_mut(key) {
                match asset {
                    AssetHandle::Lazy(path) => {
                        *asset = AssetHandle::Loaded(self.asset_server.load(path.to_owned()))
                    }
                    AssetHandle::Loaded(_) => {}
                }
            }
        })
    }

    pub fn get(&self, key: Key) -> Option<Handle<Asset>> {
        self.assets
            .write()
            .unwrap()
            .get_mut(&key)
            .map(|asset| match asset {
                AssetHandle::Lazy(path) => {
                    let handle = self.asset_server.load(path.to_owned());
                    *asset = AssetHandle::Loaded(handle.clone_weak());

                    handle
                }
                AssetHandle::Loaded(handle) => handle.clone_weak(),
            })
    }

    pub fn get_many(&self, keys: &[Key]) -> Vec<Handle<Asset>> {
        let mut lock = self.assets.write().unwrap();
        let get_asset = |key| {
            lock.get_mut(key).map(|asset| match asset {
                AssetHandle::Lazy(path) => {
                    let handle = self.asset_server.load(path.to_owned());
                    *asset = AssetHandle::Loaded(handle.clone_weak());

                    handle
                }
                AssetHandle::Loaded(handle) => handle.clone_weak(),
            })
        };

        keys.into_iter().filter_map(get_asset).collect()
    }
}
