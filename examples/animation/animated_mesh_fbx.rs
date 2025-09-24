//! Plays an animation on an FBX model of an animated cube.

use std::f32::consts::PI;

use bevy::{light::CascadeShadowConfigBuilder, prelude::*, scene::SceneInstanceReady, fbx::FbxAssetLabel};

// An example FBX asset that contains a mesh and animation.
const FBX_PATH: &str = "models/animated/cube_anim.fbx";

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_mesh_and_animation)
        .add_systems(Startup, setup_camera_and_environment)
        .add_systems(Update, debug_animation_status)
        .run();
}

// A component that stores a reference to an animation we want to play. This is
// created when we start loading the mesh (see `setup_mesh_and_animation`) and
// read when the mesh has spawned (see `play_animation_when_ready`).
#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn setup_mesh_and_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Create an animation graph containing the first animation from the FBX.
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(FbxAssetLabel::Animation(0).from_asset(FBX_PATH)),
    );

    // Store the animation graph as an asset.
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // Start loading the asset as a scene and store a reference to it in a
    // SceneRoot component. This component will automatically spawn a scene
    // containing our mesh once it has loaded.
    let mesh_scene = SceneRoot(asset_server.load(FbxAssetLabel::Scene(0).from_asset(FBX_PATH)));

    // Spawn an entity with our components, and connect it to an observer that
    // will trigger when the scene is loaded and spawned.
    commands
        .spawn((animation_to_play, mesh_scene))
        .observe(play_animation_when_ready);
}

fn play_animation_when_ready(
    event: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    // The entity we spawned in `setup_mesh_and_animation` is the trigger's target.
    // Start by finding the AnimationToPlay component we added to that entity.
    if let Ok(animation_to_play) = animations_to_play.get(event.entity()) {
        // The SceneRoot component will have spawned the scene as a hierarchy
        // of entities parented to our entity. Since the FBX contained a mesh
        // and animations, it should have spawned an animation player component.
        // Search our entity's descendants to find the animation player.
        for child in children.iter_descendants(event.entity()) {
            if let Ok(mut player) = players.get_mut(child) {
                // Tell the animation player to start the animation and keep
                // repeating it.
                player.play(animation_to_play.index).repeat();

                // Add the animation graph. This only needs to be done once to
                // connect the animation player to the mesh.
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

fn debug_animation_status(
    time: Res<Time>,
    players: Query<(&AnimationPlayer, Entity, Option<&Name>)>,
    meshes: Query<(&Transform, Entity, Option<&Name>), With<Mesh3d>>,
) {
    // Print status every second after 1 second has passed
    if time.elapsed_secs() > 1.0 && time.elapsed_secs() as u32 % 1 == 0 && time.delta_secs() < 0.1 {
        info!("=== ANIMATION DEBUG at t={:.1}s ===", time.elapsed_secs());
        
        for (player, entity, name) in players.iter() {
            let name_str = name.map(|n| n.as_str()).unwrap_or("unnamed");
            info!("AnimationPlayer at {:?} ({}): paused={}", 
                  entity, name_str, player.all_paused());
        }
        
        for (transform, entity, name) in meshes.iter() {
            let name_str = name.map(|n| n.as_str()).unwrap_or("unnamed");
            info!("Mesh at {:?} ({}): pos={:?}, rot={:?}, scale={:?}", 
                  entity, name_str, transform.translation, transform.rotation, transform.scale);
        }
    }
}

// Spawn a camera and a simple environment with a ground plane and light.
fn setup_camera_and_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera - positioned to view the animated FBX cube
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-50.0, 20.0, 50.0).looking_at(Vec3::new(-50.0, 0.0, 0.0), Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));
}