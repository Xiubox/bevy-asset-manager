# Bevy Asset Manager

This crate provides a simple asset management system for the Bevy game engine.
It defines an `AssetManager` which handles loading and retriving assets based on enum key variants,
with support for lazyiily and eagerly loading game assets. Macros are
provided for easy creation of asset managers.

## Example

``` rust
use bevy::prelude::{Commands, Component, Res, Resource, States};
use bevy_asset_manager::{mixed_asset_manager, AssetManager};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl, AudioSource};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_audio_channel::<Ship>()
            .add_state::<ShipState>()
            .add_systems(Startup, setup)
            .add_systems(Update, handle_input)
            .add_systems(OnEnter(ShipState::Idle), idle)
            .add_systems(OnEnter(ShipState::Accelerate), accelerate);
    }
}

#[derive(Component, Resource)]
struct Ship;

#[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
enum ShipState {
    #[default]
    Idle,
    Accelerate,
}

// Shorthand for our ship audio's asset manager
type ShipAudioManager = AssetManager<ShipAudio, AudioSource>;

// Keys for our ship audio
enum ShipAudio {
    EngineOn,
    EngineOff,
    Warp,
}

// Create an asset manager resource and insert it into our runtime
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(
        mixed_asset_manager!(<ShipAudio, AudioSource> binds asset_server.clone(), {
            LoadStyle::Loaded, ShipAudio::EngineOn => "sounds/engine-on.ogg",
            LoadStyle::Loaded, ShipAudio::EngineOff => "sounds/engine-off.ogg",
            LoadStyle::Lazy, ShipAudio::Warp => "sounds/warp.ogg",
        }),
    );
}

// Retrieve and use our engine on audio asset
fn accelerate(hannel: Res<AudioChannel<Ship>>, audio_manager: Res<ShipAudioManager>) {
    audio.stop();
    audio
        .play(audio_manager.get(ShipAudio::EngineOn).unwrap())
        .with_volume(0.5)
        .loop_from(1.0);
}

// Retrieve and use our engine off audio asset
fn idle(hannel: Res<AudioChannel<Ship>>, audio_manager: Res<ShipAudioManager>) {
    audio.stop();
    audio
        .play(audio_manager.get(ShipAudio::EngineOff).unwrap())
        .with_volume(0.5);
}

fn handle_input(keys: Res<Input<KeyCode>>, mut ship_state: ResMut<NextState<ShipState>>) {
    if keys.just_pressed(KeyCode::W) {
        ship_state.set(ShipState::Accelerate);
    }

    if keys.just_released(KeyCode::W) {
        ship_state.set(ShipState::Idle);
    }
}
```

## Note

This documentation assumes familiarity with Bevy's asset api and ECS framework.
Ensure that Bevy is properly integrated into your project for optimal use of this crate.

For more details on Bevy asset, refer to the Bevy documentation:
[Bevy Documentation](https://bevyengine.org/).
