mod flight;

use crate::flight::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<Phase>()
        .add_systems(Startup, setup)
        .add_systems(PostStartup, state_recon)
        .add_systems(Update, (
                move_drone_system.run_if(in_state(Phase::Recon)),
                update_quads,
        ))
        .add_systems(OnEnter(Phase::Recon), recon)
        .run();
}

#[derive(Resource)]
struct Textures {
    land: Handle<Image>,
    water: Handle<Image>,
    base: Handle<Image>,
    drone: Handle<Image>,
    grey_border: Handle<Image>,
    blue_border: Handle<Image>,
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
        base: asset_server.load("base_tile.png"),
        drone: asset_server.load("drone.png"),
        grey_border: asset_server.load("grey_border.png"),
        blue_border: asset_server.load("blue_border.png"),
    };

    let width = 60;
    let height = 30;

    // generar mapa procedural
    let map = generate_coast_map(width, height);
                           
    commands.spawn(( // Spawn camera in the center
            Camera2d,
            Transform::from_xyz(width as f32 / 2.0 * TILE_SIZE, height as f32 / 2.0 * TILE_SIZE, 10.)
    ));

    for x in 0..width {
        for y in 0..height {
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
            translation: Vec3::new(TILE_SIZE, (height as f32 * TILE_SIZE) - (2.0 * TILE_SIZE), 2.0),
            scale: Vec3::splat(TILE_SIZE * 1.8 / 128.0),
            ..Default::default()
        },
        Drone::init_with_pos(1, height - 2),
    ));

    commands.insert_resource(textures);

    commands.insert_resource(map);
}

fn state_recon(mut next_state: ResMut<NextState<Phase>>) {
    // This happens when Startup is finished
    next_state.set(Phase::Recon);
}

fn move_drone_system(
    time: Res<Time>,
    mut targets: ResMut<ReconTargets>,
    mut query: Query<(&mut Drone, &mut Transform)>,
    mut next_state: ResMut<NextState<Phase>>,
    mut commands: Commands,
) {
    // Mueve el dron hacia el target
    // Con una velocidad de drone.speed = 10px / sec
    for (mut drone, mut transform) in &mut query {
        if let Some(target) = &drone.target {
            let dir = target.quad.pos - drone.pos;
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
                // TODO
                // Target in range, next target 
                if targets.current == targets.quads.len() {
                    next_state.set(Phase::Flight);
                    commands.remove_resource::<ReconTargets>();
                    println!("Phase FLight!!");
                    break;
                }
                drone.set_target(&targets.quads[targets.current]);
                targets.current += 1;
            }
        } else { // Shouldn't happen but just in case targets.is_none()
            drone.set_target(&targets.quads[0]);
        }
    }
}

fn recon(
    map: Res<Map>,
    mut commands: Commands,
    textures: Res<Textures>,
) {
    let quads = reconnaissance(&map);
    println!("Reconocimiento completado, se mapearon {} cuadrantes", quads.len());

    for quad in quads.iter() {
        match quad.kind {
            Tile::Land => {
                commands.spawn((
                        Sprite {
                            image: textures.grey_border.clone(),
                            ..Default::default()
                        },
                        Transform {
                            translation: Vec3::new((quad.pos.x - 0.5) * TILE_SIZE, (quad.pos.y - 0.5) * TILE_SIZE, 1.0),
                            scale: Vec3::splat(TILE_SIZE / 128.0),
                            ..Default::default()
                        }
                ));
            },
            Tile::Water => {
                commands.spawn((
                        Sprite {
                            image: textures.blue_border.clone(),
                            ..Default::default()
                        },
                        Transform {
                            translation: Vec3::new((quad.pos.x - 0.5) * TILE_SIZE, (quad.pos.y - 0.5) * TILE_SIZE, 1.0),
                            scale: Vec3::splat(TILE_SIZE / 128.0),
                            ..Default::default()
                        }
                ));
            },
            _ => (),
        };
    }

    let targets = target_vector(quads.clone());

    let recon_targets = ReconTargets { quads: targets , current: 0 };

    commands.insert_resource(Quadrants { q: quads } );
    commands.insert_resource(recon_targets);

    /*
    let targets = water_targets(&quadrants);
    println!("Se encontraron {} cuadrantes de agua", targets.positions.len());

    commands.insert_resource(targets);
    */
}

fn update_quads(
    mut quads: ResMut<Quadrants>,
) {
    // Update quads when they are found and then done

}
