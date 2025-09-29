mod flight;

use crate::flight::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<Phase>()
        .add_systems(Startup, setup)
        .add_systems(PostStartup, state_waiting)
        .add_systems(Update, (
                waiting.run_if(in_state(Phase::Waiting)),
                drone_route_recon.run_if(in_state(Phase::Recon)),
                drone_route_flight.run_if(in_state(Phase::Flight)),
        ))
        .add_systems(OnEnter(Phase::Recon), recon)
        .add_systems(OnEnter(Phase::Flight), flight)
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
    checkmark: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
enum Phase {
    #[default]
    Setup,
    Waiting,
    Recon,
    Flight,
    Done,
}

const TILE_SIZE: f32 = 20.0; // px
// Map size
const WIDTH: usize = 60;
const HEIGHT: usize = 30;

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
        checkmark: asset_server.load("checkmark.png"),
    };

    // generar mapa procedural
    let map = generate_coast_map(WIDTH, HEIGHT);
                           
    commands.spawn(( // Spawn camera in the center
            Camera2d,
            Transform::from_xyz(WIDTH as f32 / 2.0 * TILE_SIZE, HEIGHT as f32 / 2.0 * TILE_SIZE, 10.)
    ));

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
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
            translation: Vec3::new(TILE_SIZE, (HEIGHT as f32 * TILE_SIZE) - (2.0 * TILE_SIZE), 2.0),
            scale: Vec3::splat(TILE_SIZE * 1.8 / 128.0),
            ..Default::default()
        },
        Drone::init_with_pos(1, HEIGHT - 2),
    ));

    commands.insert_resource(textures);

    commands.insert_resource(map);
}

fn state_waiting(
    mut next_state: ResMut<NextState<Phase>>,
) { 
    next_state.set(Phase::Waiting);
}

fn waiting(
    mut next_state: ResMut<NextState<Phase>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(Phase::Recon);
    }
}

fn drone_route_recon(
    time: Res<Time>,
    textures: Res<Textures>,
    mut targets: ResMut<Targets>,
    mut water_quads: ResMut<Quadrants>,
    mut query: Query<(&mut Drone, &mut Transform)>,
    mut next_state: ResMut<NextState<Phase>>,
    mut commands: Commands,
) {
    // Mueve el dron hacia el target
    // Con una velocidad de drone.speed = 10px / sec
    for (mut drone, mut transform) in &mut query {
        // Move drone towards it's target, returning true if reached its target.
        let in_range = drone.move_towards(&mut transform, time.delta_secs(), &targets.targets[0]);

        if in_range {
            // Check quadrant and draw its border
            if let Some(target) = &drone.target {
                let quad = &target.quad;
                let texture = match quad.kind {
                    Tile::Water => {
                        water_quads.q.push(quad.clone());
                        textures.blue_border.clone()
                    }, 
                    _ => textures.grey_border.clone(),
                };

                commands.spawn((
                    Sprite {
                        image: texture,
                        ..Default::default()
                    },
                    Transform {
                        translation: Vec3::new((quad.pos.x - 0.5) * TILE_SIZE, (quad.pos.y - 0.5) * TILE_SIZE, 1.0),
                        scale: Vec3::splat(TILE_SIZE / 128.0),
                        ..Default::default()
                    }
                ));
            }

            // If its the last quad in the recon phase, go to the next phase: flight!
            if targets.current == targets.targets.len() {
                next_state.set(Phase::Flight);
                commands.remove_resource::<Targets>();
                println!("Phase FLight!!");
                break;
            }

            // Move to next target!  yes i hate myself
            drone.set_target(&targets.targets[targets.current]);
            targets.current += 1;
        }
    }
}

fn drone_route_flight(
    time: Res<Time>,
    textures: Res<Textures>,
    mut commands: Commands,
    mut targets: ResMut<Targets>,
    mut query: Query<(&mut Drone, &mut Transform)>,
    mut next_state: ResMut<NextState<Phase>>,
) {
    // Mueve el dron hacia el target
    // Con una velocidad de drone.speed = 10px / sec
    for (mut drone, mut transform) in &mut query {
        let in_range;
        match drone.moving {
            Moving::ToBase => in_range = drone.move_towards_base(&mut transform, time.delta_secs()),
            Moving::ToQuad => in_range = drone.move_towards(&mut transform, time.delta_secs(), &targets.targets[0]),
        }

        if in_range {
            // If its the last quad in the flight phase, DONE!
            if targets.current == targets.targets.len() && drone.moving == Moving::ToQuad {
                if let Some(target) = &drone.target {
                    let quad = &target.quad;
                    commands.spawn((
                        Sprite {
                            image: textures.checkmark.clone(),
                            ..Default::default()
                        },
                        Transform {
                            translation: Vec3::new((quad.pos.x - 0.5) * TILE_SIZE, (quad.pos.y - 0.5) * TILE_SIZE, 1.0),
                            scale: Vec3::splat(TILE_SIZE / 128.0),
                            ..Default::default()
                        }
                    ));
                }

                next_state.set(Phase::Done);
                commands.remove_resource::<Targets>();
                println!("FINISHED DRONE ROUTE");
                break;
            }

            match drone.moving {
                Moving::ToBase => {
                    if targets.current > 1 {
                        if let Some(target) = &drone.target {
                            let quad = &target.quad;
                            commands.spawn((
                                Sprite {
                                    image: textures.checkmark.clone(),
                                    ..Default::default()
                                },
                                Transform {
                                    translation: Vec3::new((quad.pos.x - 0.5) * TILE_SIZE, (quad.pos.y - 0.5) * TILE_SIZE, 1.0),
                                    scale: Vec3::splat(TILE_SIZE / 128.0),
                                    ..Default::default()
                                }
                            ));
                        }
                    }

                    // Move to next target!  yes i hate myself
                    if targets.current != targets.targets.len() {
                        drone.set_target(&targets.targets[targets.current]);

                        drone.moving = Moving::ToQuad;
                    }
                }, Moving::ToQuad => {
                    drone.moving = Moving::ToBase;
                    targets.current += 1;
                },
            }
        }
    }
}

fn recon(
    map: Res<Map>,
    mut commands: Commands,
) {
    let quads = reconnaissance(&map);
    println!("Reconocimiento completado, se mapearon {} cuadrantes", quads.len());

    let targets = target_vector(quads);
    let recon_targets = Targets { targets , current: 0 };

    // commands.insert_resource(Quadrants { q: quads } );
    commands.insert_resource(recon_targets);

    let water_quads: Quadrants = Quadrants { q: vec![] };
    commands.insert_resource(water_quads);

    /*
    let targets = water_targets(&quadrants);
    println!("Se encontraron {} cuadrantes de agua", targets.positions.len());

    commands.insert_resource(targets);
    */
}

fn flight(
    water_targets: Res<Quadrants>,
    mut commands: Commands,
) {
    let targets = target_vector(water_targets.q.clone());
    println!("Fase de reconocimiento completada, se encontraron {} cuadrantes para medir", targets.len());

    let flight_targets = Targets { targets, current: 0 };
    commands.insert_resource(flight_targets);
}

