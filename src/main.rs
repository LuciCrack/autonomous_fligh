mod flight;

use crate::flight::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<Phase>()
        .add_systems(Startup, setup)
        .add_systems(Update, move_drone_system)
        .add_systems(PostStartup, state_recon)
        .add_systems(OnEnter(Phase::Recon), recon)
        .add_systems(OnEnter(Phase::Flight), autonomous)
        .run();
}

#[derive(Resource)]
struct Textures {
    land: Handle<Image>,
    water: Handle<Image>,
    water_marked: Handle<Image>,
    base: Handle<Image>,
    drone: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
enum Phase {
    #[default]
    Setup,
    Recon,
    Flight,
}

const TILE_SIZE: f32 = 20.0; // px

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Cargar texturas
    let textures = Textures {
        land: asset_server.load("land_tile.png"),
        water: asset_server.load("water_tile.png"),
        water_marked: asset_server.load("water_tile_2.png"),
        base: asset_server.load("base_tile.png"),
        drone: asset_server.load("drone.png"),
    };

    let width = 32;
    let height = 60;

    // generar mapa procedural (puedes usar generate_coast_map con ruido)
    let map = generate_coast_map(width, height);
                           
    commands.spawn(( // Spawn camera in the center
            Camera2d,
            Transform::from_xyz(height as f32 / 2.0 * TILE_SIZE, width as f32 / 2.0 * TILE_SIZE, 10.)
    ));

    for x in 0..height {
        for y in 0..width {
            // Seleccionar textura
            let tile_type = get_tile_type(&map, x, y);
            let texture = match tile_type {
                Tile::Land => textures.land.clone(),
                Tile::Water => textures.water.clone(),
                Tile::Base => textures.base.clone()
            };

            commands.spawn((
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform {
                    translation: Vec3::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        0.0,
                    ),
                    scale: Vec3::splat(TILE_SIZE / 128.0), // make 128px image smaller so it fit into screen
                    ..Default::default()
                }
            ));
        }
    }

    // Spawnear dron (encima de los tiles)
    commands.spawn((
        Sprite {
            image: textures.drone.clone(),
            ..default()
        },
        Transform {
            translation: Vec3::new(TILE_SIZE, (width as f32 * TILE_SIZE) - (2.0 * TILE_SIZE), 2.0),
            scale: Vec3::splat(TILE_SIZE * 1.8 / 128.0),
            ..Default::default()
        },
        Drone::init_with_pos(1, width - 2),
    ));

    commands.insert_resource(textures);

    commands.insert_resource(map);
}

fn state_recon(mut next_state: ResMut<NextState<Phase>>) {
    next_state.set(Phase::Recon);
}

fn move_drone_system(
    time: Res<Time>,
    // TODO
    // targets: Res<Targets>,
    mut query: Query<(&mut Drone, &mut Transform)>,
) {
    for (mut drone, mut transform) in &mut query {
        if let Some(target) = drone.target {
            let dir = target - drone.pos;
            let dist = ops::sqrt((dir.x * dir.x) + (dir.y * dir.y));

            if dist > 1.0 {
                let step = drone.speed * time.delta_secs();
                let movement = dir.normalize() * step.min(dist);

                // update logical pos
                drone.pos += movement;
                // update sprite
                transform.translation.x = drone.pos.x as f32 * TILE_SIZE;
                transform.translation.y = drone.pos.y as f32 * TILE_SIZE;
            } else {
                // reached target
                drone.target = None;
            }
        } else {
            // TODO: so I think all thats missing is making the drone move to each quadrant in the
            // recon phase, making targets as it goes.
            // Then make the drone move to each target coming back to the base.
            // but idk how to schechule that so thats for tomorrow if I want lol
            // drone.target = Some(next_target())
            //
            // For now I just like to see it move :D 
            drone.target = Some(Vec2::new(5.0, 8.0));
        }
    }
}

fn recon(
    map: Res<Map>,
    mut commands: Commands,
) {
    let quadrants = reconnaissance(&map);
    println!("Reconocimiento completado, se mapearon {} cuadrantes", quadrants.len());

    let targets = water_targets(&quadrants);
    println!("Se encontraron {} cuadrantes de agua", targets.positions.len());

    commands.insert_resource(targets);
}

fn autonomous(
    mut query: Query<&mut Drone>,
    targets: Res<Targets>,
) {
    for mut drone in query.iter_mut() {
        autonomous_flight(&mut drone, &*targets);
    }
}

