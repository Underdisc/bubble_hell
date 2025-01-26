use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::{
    color::palettes::css::*,
    math::bounding::{BoundingSphere, IntersectsVolume},
    prelude::*,
};
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;

const PLAYER_MOVEMENT_SPEED: f32 = 10.0;
const PLAYER_RADIUS: f32 = 0.35;
const PLAYER_OXYGEN_START_SUPPLY: f32 = 100.0;
const PLAYER_OXYGEN_DECREASE_PER_SECOND: f32 = 1.0;
const BUBBLE_RADIUS: f32 = 0.6; //defines size of the bubbles
const BUBBLE_SPAWN_RADIUS: f32 = 6.0; //defines the radius of the circle on which bubbles are spawned
const BUBBLE_HOVER_OFFSET: f32 = 0.25; //added to player_translation.y, so bubbles are slightly higher than player mesh; emphasizes transparency
const BUBBLE_MOVEMENT_SPEED: f32 = 0.2;
const GAME_OVER_SCREEN_DISTANCE: f32 = 1.5;

const ASSET_SCALE: f32 = 0.3; //we scale all 3D models with this because of reasons

#[derive(Event)]
struct GameOverEvent;

#[derive(Resource)]
struct IsGameOver(bool);

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
    Blood,   //Death
    Dirt,
    Freeze,
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
        .add_plugins(LogDiagnosticsPlugin::default())
        .insert_resource(BubbleSpawnTimer(Timer::from_seconds(
            0.2,
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
        .add_systems(
            Update,
            (
                on_asset_loaded,
                reduce_oxygen_level,
                play_game_over_sound,
                show_game_over_screen,
            ),
        )
        .add_event::<GameOverEvent>()
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
                                Transform::from_translation(Vec3::splat(0.0_f32))
                                    .with_scale(Vec3::splat(ASSET_SCALE)),
                            ));
                        }

                        "plateau" => {
                            commands.spawn((
                                Plateau,
                                Transform::from_translation(Vec3::splat(0.0_f32))
                                    .with_scale(Vec3::splat(ASSET_SCALE)),
                                SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                            ));
                        }

                        "bubble_rot" => {
                            bubble_models
                                .0
                                .insert(BubbleType::Blood, gltf_asset.default_scene.clone());
                        }

                        "bubble_dirt" => {
                            bubble_models
                                .0
                                .insert(BubbleType::Dirt, gltf_asset.default_scene.clone());
                        }

                        "bubble_freeze" => {
                            bubble_models
                                .0
                                .insert(BubbleType::Freeze, gltf_asset.default_scene.clone());
                        }

                        "bubble_regular" => {
                            bubble_models
                                .0
                                .insert(BubbleType::Regular, gltf_asset.default_scene.clone());
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

fn play_game_over_sound(
    asset_server: Res<AssetServer>,
    mut game_over_event_reader: EventReader<GameOverEvent>,
    mut commands: Commands,
    audio_players: Query<Entity, With<AudioPlayer>>,
) {
    for _event in game_over_event_reader.read() {
        info!("Game Over - Thanks for dying :-)");
        //despawn all running AudioPlayers
        for entity in audio_players.iter() {
            commands.entity(entity).despawn();
        }

        // spawn the game over sound
        commands.spawn(AudioPlayer::new(
            asset_server.load("background rumbling.wav"),
        ));
    }
}

fn show_game_over_screen(
    mut commands: Commands,
    mut game_over_event_reader: EventReader<GameOverEvent>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_transform: Single<&Transform, With<Camera3d>>,
) {
    let mut is_game_over = false;
    for _event in game_over_event_reader.read() {
        is_game_over = true;
    }

    if !is_game_over {
        return;
    }

    // create quad handlePrimitiveTopology::Qua, asset_usa
    let screen_mesh_handle = meshes.add(Plane3d::default());

    // create texture handle
    let texture_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("Game Over2.png")),
        perceptual_roughness: 1.0,
        ..default()
    });

    // calculate camera-attached transform & rotation
    let camera_transform = camera_transform.into_inner();
    let screen_location =
        camera_transform.translation + camera_transform.forward() * GAME_OVER_SCREEN_DISTANCE;

    info!(
        "camera rotXYZ: {:?}",
        camera_transform.rotation.to_euler(EulerRot::XYZ)
    );

    commands.spawn((
        Mesh3d(screen_mesh_handle.clone()),
        MeshMaterial3d(texture_handle.clone()),
        Transform::from_translation(screen_location).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.4,
            0.0,
            0.0,
        )),
    ));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // create a player entity and the camera
    // we need to do this in setup because the player_movement requires the an entity with
    // a player component Tag and a Transform
    let camera_direction: Vec3 = Vec3::normalize(Vec3::new(0.0, 1.0, 0.0));
    commands
        .spawn((
            Player,
            OxygenLevel(PLAYER_OXYGEN_START_SUPPLY),
            Transform::default(),
            InheritedVisibility::VISIBLE,
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 10.0, 3.0).looking_at(camera_direction, Vec3::Y),
            ));
            
            parent.spawn(
                (SpotLight {                    
                    color: GREY.into(),
                    intensity: 500_000.0,  
                    range: 5.0,
                    radius: 10.0, 
                    inner_angle: 80.0,  
                    shadows_enabled: true,       
                    ..Default::default()             
                },
                Transform::from_xyz(0.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));

            parent.spawn(
                (SpotLight {
                    intensity: 100_000.0,  
                    range: GAME_OVER_SCREEN_DISTANCE * 2.0,
                    radius: 10.0, 
                    inner_angle: 1.0,                                    
                    outer_angle: 100.0,
                    ..Default::default()
                },
                Transform::from_xyz(0.0, 10.0, 3.0).looking_at(camera_direction, Vec3::Y),
            ));
             
        });

    // create light
    
    commands.insert_resource(AmbientLight {
        color: ROYAL_BLUE.into(),
        brightness: 100.0,
    }); 
   
    commands.insert_resource(IsGameOver(false));

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
    commands.spawn(AudioPlayer::new(asset_server.load("Music.ogg")));

    commands.spawn(AudioPlayer::new(
        asset_server.load("Stereotypische unterwasser Atmo.mp3"),
    ));
}

fn reduce_oxygen_level(
    mut oxygen_level: Single<&mut OxygenLevel>,
    time: Res<Time>,
    mut game_over_event_writer: EventWriter<GameOverEvent>,
    mut is_game_over: ResMut<IsGameOver>,
) {
    if oxygen_level.0 <= 0.0_f32 {
        return;
    }

    oxygen_level.0 = oxygen_level.0 - (time.delta_secs() * PLAYER_OXYGEN_DECREASE_PER_SECOND);
    info!("remaining oxygen: {}", oxygen_level.0);

    if !is_game_over.0 && oxygen_level.0 <= 0.0_f32 {
        game_over_event_writer.send(GameOverEvent {});
        is_game_over.0 = true;
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
    is_game_over: Res<IsGameOver>,
) {
    if is_game_over.0 {
        return;
    }

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
        movement = time.delta_secs() * PLAYER_MOVEMENT_SPEED * Vec2::normalize(movement);
        player_transform.translation.x += movement.x;
        player_transform.translation.z += movement.y;
    }
}

fn bubble_spawns(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<BubbleSpawnTimer>,
    bubble_models: Res<BubbleModels>,
    player_transform: Single<&Transform, With<Player>>,
    is_game_over: Res<IsGameOver>,
) {
    if is_game_over.into_inner().0 {
        return;
    }

    //do not run until all models are loaded
    let mut rng = rand::thread_rng();

    //randomly decide bubble type
    let bubble_type = match rng.gen_range(0..4) {
        0 => BubbleType::Regular,
        1 => BubbleType::Blood,
        2 => BubbleType::Dirt,
        3 => BubbleType::Freeze,
        _ => BubbleType::Regular,
    };

    if bubble_models.0.get(&bubble_type).is_none() {
        warn!("no model loaded for bubble type {:?}", &bubble_type);
        //just don't spawn until all models are loaded
        return;
    }

    if timer.0.tick(time.delta()).just_finished() {
        let player_translation = player_transform.into_inner().translation;
        let random_rotation = rng.gen::<f32>();
        let rotation_vector = Rot2::degrees(random_rotation * 360.0);

        // generate random position on edge of circle around player transform
        let spawn_location = Vec3::from_array([
            player_translation.x + rotation_vector.cos * BUBBLE_SPAWN_RADIUS,
            player_translation.y + BUBBLE_HOVER_OFFSET,
            player_translation.z + rotation_vector.sin * BUBBLE_SPAWN_RADIUS,
        ]);

        // calculate movement angle directly at player
        let bubble_movement_direction = Vec2::from([
            (player_translation.x - spawn_location.x) * BUBBLE_MOVEMENT_SPEED,
            (player_translation.z - spawn_location.z) * BUBBLE_MOVEMENT_SPEED,
        ]);

        commands.spawn((
            Transform::from_translation(spawn_location).with_scale(Vec3::splat(BUBBLE_RADIUS)),
            Bubble,
            Velocity(bubble_movement_direction),
            SceneRoot(bubble_models.0.get(&bubble_type).unwrap().clone().unwrap()),
        ));
    }
}

fn move_bubbles(
    mut bubble_query: Query<(&mut Transform, &Velocity), With<Bubble>>,
    time: Res<Time>,
) {
    //note: bubbles move on the x-z-plane; with x pointing right and z pointing up
    for (mut transform, velocity) in &mut bubble_query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.z += velocity.0.y * time.delta_secs();
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
        }
    }
}
