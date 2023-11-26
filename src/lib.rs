//! # Bevy Asset Manager
//!
//! This crate provides a simple asset management system for the Bevy game engine.
//! It defines an `AssetManager` which handles loading and retriving assets based on enum key variants,
//! with support for lazyiily and eagerly loading game assets. Macros are
//! provided for easy creation of asset managers.
//!
//! # Example
//!
//! ```edition2021
//! use bevy::prelude::{Commands, Component, Res, Resource, States};
//! use bevy_asset_manager::{mixed_asset_manager, AssetManager};
//! use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl, AudioSource};
//!
//! pub struct ShipPlugin;
//!
//! impl Plugin for ShipPlugin {
//!     fn build(&self, app: &mut bevy::prelude::App) {
//!         app.add_audio_channel::<Ship>()
//!             .add_state::<ShipState>()
//!             .add_systems(Startup, setup)
//!             .add_systems(Update, handle_input)
//!             .add_systems(OnEnter(ShipState::Idle), idle)
//!             .add_systems(OnEnter(ShipState::Accelerate), accelerate);
//!     }
//! }
//!
//! #[derive(Component, Resource)]
//! struct Ship;
//!
//! #[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
//! enum ShipState {
//!     #[default]
//!     Idle,
//!     Accelerate,
//! }
//!
//! // Shorthand for our ship audio's asset manager
//! type ShipAudioManager = AssetManager<ShipAudio, AudioSource>;
//!
//! // Keys for our ship audio
//! enum ShipAudio {
//!     EngineOn,
//!     EngineOff,
//!     Warp,
//! }
//!
//! // Create an asset manager resource and insert it into our runtime
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     commands.insert_resource(
//!         mixed_asset_manager!(<ShipAudio, AudioSource> binds asset_server.clone(), {
//!             LoadStyle::Loaded, ShipAudio::EngineOn => "sounds/engine-on.ogg",
//!             LoadStyle::Loaded, ShipAudio::EngineOff => "sounds/engine-off.ogg",
//!             LoadStyle::Lazy, ShipAudio::Warp => "sounds/warp.ogg",
//!         }),
//!     );
//! }
//!
//! // Retrieve and use our engine on audio asset
//! fn accelerate(hannel: Res<AudioChannel<Ship>>, audio_manager: Res<ShipAudioManager>) {
//!     audio.stop();
//!     audio
//!         .play(audio_manager.get(ShipAudio::EngineOn).unwrap())
//!         .with_volume(0.5)
//!         .loop_from(1.0);
//! }
//!
//! // Retrieve and use our engine off audio asset
//! fn idle(hannel: Res<AudioChannel<Ship>>, audio_manager: Res<ShipAudioManager>) {
//!     audio.stop();
//!     audio
//!         .play(audio_manager.get(ShipAudio::EngineOff).unwrap())
//!         .with_volume(0.5);
//! }
//!
//! fn handle_input(keys: Res<Input<KeyCode>>, mut ship_state: ResMut<NextState<ShipState>>) {
//!     if keys.just_pressed(KeyCode::W) {
//!         ship_state.set(ShipState::Accelerate);
//!     }
//!
//!     if keys.just_released(KeyCode::W) {
//!         ship_state.set(ShipState::Idle);
//!     }
//! }
//! ```
//!
//! ## Note
//!
//! This documentation assumes familiarity with Bevy's asset api and ECS framework.
//! Ensure that Bevy is properly integrated into your project for optimal use of this crate.
//!
//! For more details on Bevy asset, refer to the Bevy documentation:
//! [Bevy Documentation](https://bevyengine.org/).

use bevy::{
    prelude::{AssetServer, Handle, Resource},
    utils::hashbrown::HashMap,
};
use std::{hash::Hash, sync::RwLock};

/// Creates an `AssetManager<$key_kind, $asset_kind>` with unloaded assets.
///
/// # Example
///
/// ```rust
/// use bevy_asset_manager::{AssetManager, lazy_asset_manager};
/// use bevy_kira_audio::AudioSource;
///
/// enum Audio {
///    EngineOn,
///    EngineOff,
///    EngineStall,
/// }
///
/// // Create a lazy asset manager with unloaded assets
/// let lazy_manager = lazy_asset_manager!(<Audio, Texture> binds asset_server.clone(), {
///     Audio::EnginOn => "sound/engine-on.ogg",
///     Audio::EnginOff => "sound/engine-off.ogg",
///     Audio::EngineStall => "sound/engine-stall.ogg",
/// });
/// ```
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

/// Creates an `AssetManager<$key_kind, $asset_kind>` with loaded assets.
///
/// # Example
///
/// ```rust
/// use bevy_asset_manager::{AssetManager, lazy_asset_manager};
/// use bevy_kira_audio::AudioSource;
///
/// enum EngineAudio {
///    EngineOn,
///    EngineOff,
///    EngineStall,
/// }
///
/// // Create a lazy asset manager with unloaded assets
/// let lazy_manager = loaded_asset_manager!(<Audio, AudioSource> binds asset_server.clone(), {
///     Audio::EngineOn => "sound/engine-on.ogg",
///     Audio::EngineOff => "sound/engine-off.ogg",
///     Audio::EngineStall => "sound/engine-stall.ogg",
/// });
/// ```
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

/// Creates an `AssetManager<$key_kind, $asset_kind>` with a combination of loaded and unloaded assets.
///
/// # Example
///
/// ```rust
/// use bevy_asset_manager::{AssetManager, lazy_asset_manager};
/// use bevy_kira_audio::AudioSource;
///
/// enum EngineAudio {
///    EngineOn,
///    EngineOff,
///    EngineStall,
/// }
///
/// // Create a lazy asset manager with unloaded assets
/// let lazy_manager = mixed_asset_manager!(<Audio, AudioSource> binds asset_server.clone(), {
///     LoadStyle::Loaded, Audio::EngineOn => "sound/engine-on.ogg",
///     LoadStyle::Loaded, Audio::EngineOff => "sound/engine-off.ogg",
///     LoadStyle::Lazy, Audio::EngineStall => "sound/engine-stall.ogg",
/// });
/// ```
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

/// The load style of an asset used in `mixed_asset_manager!` to determine if an asset should be loaded eagerly or lazily.
#[derive(Debug)]
pub enum LoadStyle {
    /// Lazily load the asset.
    Lazy,
    /// Eagerly load the asset.
    Loaded,
}

/// Enum representing different states of an asset handle.
enum AssetHandle<Asset>
where
    Asset: bevy::asset::Asset,
{
    /// Represents a lazy asset handle with the path.
    Lazy(String),
    /// Represents a loaded asset handle.
    Loaded(Handle<Asset>),
}

/// Resource representing the asset manager.
#[derive(Resource)]
pub struct AssetManager<Key, Asset>
where
    Key: PartialEq + Eq + Hash,
    Asset: bevy::asset::Asset,
{
    assets: RwLock<HashMap<Key, AssetHandle<Asset>>>,
    asset_server: AssetServer,
}

impl<Key, Asset> AssetManager<Key, Asset>
where
    Key: PartialEq + Eq + Hash + Copy,
    Asset: bevy::asset::Asset,
{
    /// Creates a new `AssetManager` instance.
    pub fn new(asset_server: AssetServer) -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
            asset_server,
        }
    }

    /// Inserts a lazy asset into the manager.
    pub fn insert(&self, key: Key, path: &str) {
        self.assets
            .write()
            .unwrap()
            .insert(key, AssetHandle::Lazy(path.to_owned()));
    }

    /// Inserts multiple lazy assets into the manager.
    pub fn insert_many(&self, pairs: &[(Key, &str)]) {
        let mut lock = self.assets.write().unwrap();

        pairs.iter().for_each(|(key, path)| {
            lock.insert(*key, AssetHandle::Lazy(path.to_owned().to_owned()));
        });
    }

    /// Inserts a loaded asset into the manager.
    pub fn insert_loaded(&self, key: Key, path: &str) {
        self.assets.write().unwrap().insert(
            key,
            AssetHandle::Loaded(self.asset_server.load(path.to_owned())),
        );
    }

    /// Inserts multiple loaded assets into the manager.
    pub fn insert_many_loaded(&self, pairs: &[(Key, &str)]) {
        let mut lock = self.assets.write().unwrap();

        pairs.iter().for_each(|(key, path)| {
            lock.insert(
                *key,
                AssetHandle::Loaded(self.asset_server.load(path.to_owned().to_owned())),
            );
        });
    }

    /// Loads an asset if it was added lazily, doing nothing if it is already loaded.
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

    /// Loads multiple assets if they were added lazily, doing nothing if they are already loaded.
    pub fn load_many(&self, keys: &[Key]) {
        let mut lock = self.assets.write().unwrap();

        keys.iter().for_each(|key| {
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

    /// Gets a handle to a loaded asset, ensuring it's loaded if it was added lazily.
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

    /// Gets multiple handles to loaded assets, ensuring they're loaded if they were added lazily.
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

        keys.iter().filter_map(get_asset).collect()
    }
}
