use bevy::window::CompositeAlphaMode;
use bevy::window::WindowLevel;
use bevy::window::WindowMode::BorderlessFullscreen;
use bevy::{prelude::*, window::CursorOptions};
use rand::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::NONE)) // Use a transparent window
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Set the window to always on top and borderlessfullscreen
                transparent: true,
                decorations: false,
                resolution: (1920, 1080).into(),
                window_level: WindowLevel::AlwaysOnTop,
                fit_canvas_to_parent: true,
                mode: BorderlessFullscreen(MonitorSelection::Current),
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(GreetTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Startup, toggle_mouse_passthrough) // This sets the mouse passthrough, theres probably an easier way to do this but im too lazy
        .add_systems(Update, execute_animations)
        .add_systems(
            Update,
            (
                // trigger the skeleton chance, its called rightsprite because i was too lazy to rename it from the example
                trigger_animation::<RightSprite>,
            ),
        )
        .run();
}

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

#[derive(Resource)]
struct GreetTimer(Timer);

#[derive(Component)]
struct RightSprite;

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // UI camera
    commands.spawn(Camera2d);
    // Load the sprite sheet using the `AssetServer`
    let texture = asset_server.load("spritesheet.png");

    // The sprite sheet is loaded here
    let layout = TextureAtlasLayout::from_grid(UVec2::new(1920, 1080), 6, 6, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let mut windowpos = WindowPosition::new(ivec2(0, 0));
    // set the animation to 50fps with the first 34 frames
    let animation_config_2 = AnimationConfig::new(0, 33, 50);
    WindowPosition::set(&mut windowpos, ivec2(0, 0));
    // Create the sprite (invisible at first but plays when you hit the chance)
    commands.spawn((
        Sprite {
            image: texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config_2.first_sprite_index,
            }),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(1.0)).with_translation(Vec3::new(0.0, 0.0, 0.0)),
        RightSprite,
        animation_config_2,
    ));
}
// self explanitory
fn toggle_mouse_passthrough(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.hit_test = false;
}

fn trigger_animation<S: Component>(
    mut animation: Single<&mut AnimationConfig, With<S>>,
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
) {
    // dont do as i do
    let mut rng = rand::rng();
    // i had to ask for help in the discord for this one
    if timer.0.tick(time.delta()).just_finished() {
        let num = rng.random_range(0..1);
        println!("{}", num);
        println!("just finished!");

        if num == 0 {
            animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
        };
    }
}

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = config.first_sprite_index;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}
