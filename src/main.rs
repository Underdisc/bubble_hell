use bevy::{
    math::bounding::{BoundingSphere, IntersectsVolume},
    prelude::*,
};
use std::collections::HashMap;
use std::collections::HashSet;

const PLAYER_RADIUS: f32 = 0.5;
const BUBBLE_RADIUS: f32 = 0.1;
static WALL_X_OFFSET: f32 = 2.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Bubble;

#[derive(Component)]
struct Environment;

#[derive(Resource)]
struct BubbleSpawnTimer(Timer);

#[derive(Resource)]
struct BubbleResource(Mesh3d, MeshMaterial3d<StandardMaterial>);

#[derive(Resource)]
struct AssetsLoading(HashMap<String, Handle<Gltf>>);

#[derive(Component)]
struct Background;

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


fn on_asset_loaded (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_assets: Res<Assets<Gltf>>,
    assets_loading: ResMut<AssetsLoading>,
    player_entity: Single<Entity, With<Player>>
)
{
    let assets_loading = assets_loading.into_inner();
    if !assets_loading.0.is_empty()
    {
        let mut processed_assets: HashSet<String> = HashSet::from([]);

        for gltf_handle in assets_loading.0.iter()
        {
            if asset_server.is_loaded_with_dependencies(gltf_handle.1.id()) 
            {
                info!("spawning asset: {}", gltf_handle.0);

                let loaded_asset =  gltf_assets.get(gltf_handle.1.id());

                if loaded_asset.is_some()
                {
                    let gltf_asset = loaded_asset.unwrap();                   
                    
                    let asset_name = gltf_handle.0.to_string();
                    match asset_name.as_str() {

                        "player_character" => { 
                            let player_character_id = commands.
                            spawn((
                                SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                                Transform::from_scale(Vec3::splat(0.3_f32)),
                            )).id();

                            commands
                            .entity(*player_entity)
                            .add_child(player_character_id);                        
                        },

                        "alge" => { 
                            for n in 0..12
                            {
                                commands.spawn((
                                    Environment,
                                    SceneRoot(gltf_asset.default_scene.clone().unwrap()),
                                    Transform::from_xyz(
                                        0.0_f32 + (n % 4) as f32,                                         
                                        0.0_f32,
                                        0.0_f32 + (n % 3) as f32, 
                                    ).with_scale(Vec3::splat(0.3_f32)),
                                ));
                            }
                        },

                        "sand_red" => {
                            commands.spawn(
                                (
                                    Background,
                                    Transform::from_xyz(0.0_f32, 0.0_f32, 0.0_f32),

                                ));
                            },

                        _ => warn!("asset name was mepty")
                    };
        
                    info!("asset {} spawned", gltf_handle.0);
                    processed_assets.insert(asset_name);
                }
                else {
                    warn!("asset {} was none", gltf_handle.0);
                }                        
            } 
        }  

        for gltf_handle in processed_assets
        {
            assets_loading.0.remove(&gltf_handle);
            info!("asset {} processed and removed from loading set", gltf_handle);
        }
        
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let wall_mesh = Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 5.0)));
    let wall_material = MeshMaterial3d(materials.add(Color::WHITE));
    commands.spawn((
        wall_mesh.clone(),
        wall_material.clone(),
        Transform::from_xyz(WALL_X_OFFSET, 0.0, 0.0),
    ));

    commands.spawn((
        wall_mesh.clone(),
        wall_material.clone(),
        Transform::from_xyz(-WALL_X_OFFSET, 0.0, 0.0),
    ));

    let camera_direction: Vec3 = Vec3::normalize(Vec3::new(0.0, -1.0, 1.0));

    commands.insert_resource(BubbleResource(
        Mesh3d(meshes.add(Sphere::new(BUBBLE_RADIUS))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 0.5, 0.7))),
    ));

    // create a player entity and the camera
    // we need to do this in setup because the player_movement requires the an entity with 
    // a player component Tag and a Transform
    commands
    .spawn((
        Player,
        Transform::default()
    ))
    .with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 3.0).looking_at(camera_direction, Vec3::Y),
        ));
    });

    info!("init loading player character...");

    commands.insert_resource(AssetsLoading(
        HashMap::from(            
            [
                ("player_character".into(), asset_server.load("Player.glb")),
                ("alge".into(), asset_server.load("Alge.glb")),
                ("sand_red".into(), asset_server.load("Sand_red.png")),
            ]
        )
    ));

    info!("player character should load now...");
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
    bubble_res: Res<BubbleResource>,
    time: Res<Time>,
    mut timer: ResMut<BubbleSpawnTimer>,
) {
    let slide_time = 5.0;
    let section_length = 5.0;
    let phase = time.elapsed().as_secs_f32() / slide_time;
    let bubble_spawn_z_offset = (phase - phase.floor()) * section_length;
    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            bubble_res.0.clone(),
            bubble_res.1.clone(),
            Transform::from_xyz(-WALL_X_OFFSET, 0.5, 2.5 - bubble_spawn_z_offset),
            Bubble,
            Velocity(Vec2::new(1.0, 0.0)),
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
        }
    }
}
