use bevy::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, player_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 5.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(-2.0, 0.0, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 5.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(2.0, 0.0, 0.0),
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
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut movement: Vec2 = Vec2::new(0.0, 0.0);
    if keyboard_input.pressed(KeyCode::KeyE) {
        movement += Vec2::new(0.0, 1.0);
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += Vec2::new(0.0, -1.0);
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
