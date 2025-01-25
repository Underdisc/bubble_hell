use bevy::{
    color::palettes::css::*, math::bounding::{BoundingSphere, IntersectsVolume}, prelude::*
};
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;

const PLAYER_RADIUS: f32 = 0.5;
const BUBBLE_RADIUS: f32 = 0.7;
static WALL_X_OFFSET: f32 = 2.0;
const ASSET_SCALE: f32 = 0.3; //we scale all 3D models with this because of reasons

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Bubble;

#[derive(Component)]
struct Environment;

#[derive(Component)]
struct OxygenLevel(f32);

#[derive(Resource)]
struct BubbleSpawnTimer(Timer);

#[derive(Resource)]
struct AssetsLoadingGltf(HashMap<String, Handle<Gltf>>);

#[derive(Debug, PartialEq, Eq, Hash)]
//the derive above is needed so we can use the enum as a key in the HashMap
//Debug is for logging
enum BubbleType {
    Regular, //Oxygon
    Blood, //Death
    Dirt,
    Freeze
}

#[derive(Resource)]
struct BubbleModels(HashMap<BubbleType, Option<Handle<Scene>>>);

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Plateau;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(BubbleSpawnTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                bubble_spawns,
                move_bubbles,
                player_movement,
                check_collisions,
            )
                .chain(),
        )
        .add_systems(Update, on_asset_loaded)
        .run();
}

fn on_asset_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_assets: Res<Assets<Gltf>>,
    assets_loading: ResMut<AssetsLoadingGltf>,
    player_entity: Single<Entity, With<Player>>,
    mut bubble_models: ResMut<BubbleModels>,
) {
    let assets_loading = assets_loading.into_inner();
    if !assets_loading.0.is_empty() {
        let mut processed_assets: HashSet<String> = HashSet::from([]);

        for gltf_handle in assets_loading.0.iter() {
            if asset_server.is_loaded_with_dependencies(gltf_handle.1.id()) {
                info!("handling loaded asset: {}", gltf_handle.0);

                let loaded_asset = gltf_assets.get(gltf_handle.1.id());

                if loaded_asset.is_some() {
                    let gltf_asset = loaded_asset.unwrap();

                    let asset_name = gltf_handle.0.to_string();
                    match asset_name.as_str() {
                        "player_character" => {
                            let player_character_id = commands
                                .spawn((
                                    SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                                    Transform::from_scale(Vec3::splat(ASSET_SCALE)),
                                ))
                                .id();

                            commands
                                .entity(*player_entity)
                                .add_child(player_character_id);
                        }

                        "alge" => {
                            for n in 0..12 {
                                commands.spawn((
                                    Environment,
                                    SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                                    Transform::from_xyz(
                                        -3.0_f32 + (n % 4) as f32,
                                        0.0_f32,
                                        -3.0_f32 + (n % 3) as f32,
                                    )
                                    .with_scale(Vec3::splat(ASSET_SCALE)),
                                ));
                            }
                        }

                        "sand" => {                         
                            commands.spawn((
                                Background,
                                SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                                Transform::from_translation(Vec3::splat(0.0_f32)).with_scale(Vec3::splat(ASSET_SCALE))                             
                            ));
                        },

                        "plateau" => {
                            commands.spawn((
                                Plateau,
                                Transform::from_translation(Vec3::splat(0.0_f32)).with_scale(Vec3::splat(ASSET_SCALE)),
                                SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                            ));
                        }

                        "bubble_rot" => {
                            bubble_models.0.insert(BubbleType::Blood, gltf_asset.default_scene.clone());
                        }

                        "bubble_dirt" => {
                            bubble_models.0.insert(BubbleType::Dirt, gltf_asset.default_scene.clone());
                        }

                        "bubble_freeze" => {
                            bubble_models.0.insert(BubbleType::Freeze, gltf_asset.default_scene.clone());
                        }

                        "bubble_regular" => {
                            bubble_models.0.insert(BubbleType::Regular, gltf_asset.default_scene.clone());
                        }

                        _ => warn!("asset name was mepty"),
                    };

                    info!("asset {} spawned", gltf_handle.0);
                    processed_assets.insert(asset_name);
                } else {
                    warn!("asset {} was none", gltf_handle.0);
                }
            }
        }

        for gltf_handle in processed_assets {
            assets_loading.0.remove(&gltf_handle);
            info!(
                "asset {} processed and removed from loading set",
                gltf_handle
            );
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {     
    // create a player entity and the camera
    // we need to do this in setup because the player_movement requires the an entity with
    // a player component Tag and a Transform
    let camera_direction: Vec3 = Vec3::normalize(Vec3::new(0.0, -1.0, 1.0));
    commands
        .spawn((Player, OxygenLevel(20.0_f32), Transform::default()))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 5.0, 3.0).looking_at(camera_direction, Vec3::Y),
            ));
        });

    // create light
    commands.insert_resource(AmbientLight {
            color: WHITE.into(),
            brightness: 1000.0,
        });

    info!("init loading assets...");

    //store material mapping for the bubbles
    commands.insert_resource(BubbleModels(HashMap::from([])));

    //load gltF files
    commands.insert_resource(AssetsLoadingGltf(HashMap::from([
        ("player_character".into(), asset_server.load("Player.glb")),
        ("alge".into(), asset_server.load("Alge.glb")),
        ("sand".into(), asset_server.load("Sand.glb")),
        ("plateau".into(), asset_server.load("Plateau.glb")),    
        ("bubble_rot".into(), asset_server.load("Bubble Rot.glb")),    
        ("bubble_dirt".into(), asset_server.load("Bubble Dirt.glb")),    
        ("bubble_freeze".into(), asset_server.load("Bubble Freeze.glb")),    
        ("bubble_regular".into(), asset_server.load("Bubble Regular.glb")), 
    ])));

    info!("player character should load now...");

    //play music
    commands.spawn(AudioPlayer::new(
        asset_server.load("Music.ogg"),
    ));

    commands.spawn(AudioPlayer::new(
        asset_server.load("Stereotypische unterwasser Atmo.mp3"),
    ));
}

fn reduce_oxygen_level(
    mut oxygen_level: Single<&OxygenLevel>,
    time: Res<Time>
)
{
    time.delta_secs();
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut movement: Vec2 = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::KeyE) {
        movement += Vec2::new(0.0, -1.0);
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += Vec2::new(0.0, 1.0);
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement += Vec2::new(-1.0, 0.0);
    }
    if keyboard_input.pressed(KeyCode::KeyF) {
        movement += Vec2::new(1.0, 0.0);
    }
    if Vec2::length_squared(movement) > 0.0 {
        let speed = 1.0;
        movement = time.delta_secs() * speed * Vec2::normalize(movement);
        player_transform.translation.x += movement.x;
        player_transform.translation.z += movement.y;
    }
}

fn bubble_spawns(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<BubbleSpawnTimer>,
    bubble_models: Res<BubbleModels>,
) {
    
    let mut rng = rand::thread_rng();
    let bubble_type = match rng.gen_range(0..4)
    {
        0 => {BubbleType::Regular}
        1 => {BubbleType::Blood}
        2 => {BubbleType::Dirt}
        3 => {BubbleType::Freeze}
        _ => {BubbleType::Regular}

    };

    if bubble_models.0.get(&bubble_type).is_none()
    {
        warn!("no model loaded for bubble type {:?}", &bubble_type);
    }

    let slide_time = 5.0;
    let section_length = 5.0;
    let phase = time.elapsed().as_secs_f32() / slide_time;
    let bubble_spawn_z_offset = (phase - phase.floor()) * section_length;

    if timer.0.tick(time.delta()).just_finished() {        

        commands.spawn((
            //bubble_res.0.clone(),
            //bubble_res.1.clone(),
            Transform::from_xyz(-WALL_X_OFFSET, 0.5, 2.5 - bubble_spawn_z_offset)
            .with_scale(Vec3::splat(BUBBLE_RADIUS)),
            Bubble,
            Velocity(Vec2::new(1.0, 0.0)),
            SceneRoot(
                bubble_models.0.get(&bubble_type).unwrap().clone().unwrap()
            ),            
        ));
    }
}

fn move_bubbles(
    mut bubble_query: Query<(&mut Transform, &Velocity), With<Bubble>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut bubble_query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn check_collisions(
    mut commands: Commands,
    player_query: Single<&Transform, With<Player>>,
    bubble_query: Query<(Entity, &Transform), With<Bubble>>,
) {
    let player_transform = player_query.into_inner();
    let player_sphere = BoundingSphere::new(player_transform.translation, PLAYER_RADIUS);
    for (bubble_entity, bubble_transform) in &bubble_query {
        let bubble_sphere = BoundingSphere::new(bubble_transform.translation, BUBBLE_RADIUS);
        if bubble_sphere.intersects(&player_sphere) {
            commands.entity(bubble_entity).despawn();
            //TODO play sound effect
        }
    }
}
