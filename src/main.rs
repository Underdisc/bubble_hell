use bevy::prelude::*;

const BUBBLE_RADIUS: f32 = 0.2;
static WALL_X_OFFSET: f32 = 2.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Bubble;

#[derive(Resource)]
struct BubbleSpawnTimer(Timer);

#[derive(Resource)]
struct BubbleResource(Mesh3d, MeshMaterial3d<StandardMaterial>);

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
            (bubble_spawns, move_bubbles, player_movement).chain(),
        )
        .run();
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

        commands
        .spawn((
            Player,
            SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0)
                    .from_asset("Player.glb"),
                ),
            ),
            Transform::from_scale(Vec3::from((0.1_f32, 0.1_f32, 0.1_f32)))
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 20.0, 3.0).looking_at(camera_direction, Vec3::Y),
            ));
        });

    commands.insert_resource(BubbleResource(
        Mesh3d(meshes.add(Sphere::new(BUBBLE_RADIUS))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.0, 0.2, 0.7))),
    ));
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
            Transform::from_xyz(WALL_X_OFFSET, 0.5, 2.5 - bubble_spawn_z_offset), Bubble,
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
