use bevy::prelude::*;
use crate::{TILE_SIZE, HEIGHT};

// Constante que define el tamaño de cada cuadrante en metros
const QUADRANT_SIZE: f32 = 10.0; // meters
// Not used for now but might be usefull later :D
const BASE: Vec2 = Vec2::new(1.0, HEIGHT as f32 - 2.0);
const SPEED: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Water,
    Land,
    Base,
}

#[derive(Debug, Resource, PartialEq)]
pub enum Moving {
    ToBase,
    ToQuad,
}

#[derive(Debug, Clone)]
pub struct Quadrant {
    pub pos: Vec2,
    pub kind: Tile,
}

#[derive(Resource)]
pub struct Quadrants { // This is really just a wrapper around a vector of Quadrants so we can use
                       // the derive(Resourse) 
    pub q: Vec<Quadrant>,
}

#[derive(Debug, Clone)]
pub struct Target { // General Target Object
    pub quad: Quadrant,
}

pub fn target_vector(quads: Vec<Quadrant>) -> Vec<Target> {
    let mut targets: Vec<Target> = vec![];
    for quad in quads {
        targets.push(Target { quad });
    }
    targets
}

#[derive(Resource)]
pub struct Targets { // List of targets
    pub targets: Vec<Target>,
    pub current: usize,
}

pub fn reconnaissance(map: &Map) -> Vec<Quadrant> {
    let width = map.tiles.len();
    let height = map.tiles[0].len();

    let quad_x = width as f32 / QUADRANT_SIZE;
    let quad_y = height as f32 / QUADRANT_SIZE;

    let mut quads = vec![];

    for x in 0..quad_x as usize {
        for y in 0..quad_y as usize {
            let kind = evaluate_quadrant(map, x, y);
            quads.push(Quadrant { 
                pos: Vec2::new(x as f32 * QUADRANT_SIZE + (QUADRANT_SIZE / 2.0), y as f32 * QUADRANT_SIZE + (QUADRANT_SIZE / 2.0)),
                kind,
            });
        }
    }

    quads
}

/*
pub fn water_targets(quadrants: &Vec<Quadrant>) -> Targets {
    let mut targets = vec![];

    for q in quadrants.iter() {
        if q.kind == Tile::Water {
            targets.push(Target { quad: q.clone() });
        }
    }

    Targets {
        target_list: targets
    }
}
*/

#[derive(Debug, Component)]
pub struct Drone {
    pub pos: Vec2,
    pub target: Option<Target>,
    pub speed: f32,
    pub moving: Moving,
    // battery: f32,
}

impl Drone {
    // Move drone cleanly towards its target
    // Return true if its in range
    pub fn move_towards(&mut self, transform: &mut Transform, delta_secs: f32, target_default: &Target) -> bool {
        if let Some(target) = &self.target {
            let dir = target.quad.pos - self.pos;
            let dist = ops::sqrt((dir.x * dir.x) + (dir.y * dir.y));

            if dist > 1.0 {
                let step = self.speed * delta_secs;
                let movement = dir.normalize() * step.min(dist);

                // update logical pos
                self.pos += movement;
                // update sprite
                transform.translation.x = self.pos.x as f32 * TILE_SIZE;
                transform.translation.y = self.pos.y as f32 * TILE_SIZE;

                // Drone not in range of it's target
                false
            } else {
                // Drone within 1.0 unit distance of its target!!
                true
            }
        } else {
            self.set_target(target_default);
            false
        }
    }

    pub fn move_towards_base(&mut self, transform: &mut Transform, delta_secs: f32) -> bool {
        let dir = BASE - self.pos;
        let dist = ops::sqrt((dir.x * dir.x) + (dir.y * dir.y));

        if dist > 1.0 {
            let step = self.speed * delta_secs;
            let movement = dir.normalize() * step.min(dist);

            // update logical pos
            self.pos += movement;
            // update sprite
            transform.translation.x = self.pos.x as f32 * TILE_SIZE;
            transform.translation.y = self.pos.y as f32 * TILE_SIZE;

            // Drone not in range of it's target
            false
        } else {
            // Drone within 1.0 unit distance of its target!!
            true
        }

    }            

    /*
    pub fn init() -> Drone {
        Drone {
            pos: Vec2::new(0.0, 0.0),
            target: None,
            speed: SPEED, // px per second
            // battery: 100.0,
        }
    }
    */
    pub fn init_with_pos(x: usize, y: usize) -> Drone {
        Drone {
            pos: Vec2::new(x as f32, y as f32),
            target: None,
            speed: SPEED,
            moving: Moving::ToQuad,
        }
    }

    pub fn set_target(&mut self, target: &Target) {
        self.target = Some(target.clone());
    }
    /*
    fn move_to(&mut self, x: usize, y: usize) {
        self.pos = Vec2::new(x as f32, y as f32);
        if self.pos == BASE {
            println!("Drone back to base at (0,0)");
        } else {     
            println!("Drone moved to: ({x}, {y})");
        }
    }

    fn move_towards(&mut self, target: &Vec2) -> bool {
        if Self::in_range(&self.pos, &target) { // En un radio de 1 metro
            if self.pos == BASE {
                println!("Drone is back at base");
            }
            return true; // ya está en rango
        }

        if self.pos.x < target.x {
            self.pos.x += 1.;
        } else if self.pos.x > target.x {
            self.pos.x -= 1.;
        }
        if self.pos.y < target.y {
            self.pos.y += 1.;
        } else if self.pos.y > target.y {
            self.pos.y -= 1.;
        }

        false
    }

    fn go_to(&mut self, target: &Vec2) {
        while !self.move_towards(target) {
            print!(".")
        }
        println!("\nDron llegó a {:?}", self.pos);
    }

    fn go_to_base(&mut self) {
        self.go_to(&BASE);
    }*/
}

#[derive(Resource)]
pub struct Map {
    tiles: Vec<Vec<Tile>>
}

pub fn generate_coast_map(width: usize, height: usize) -> Map {
    // Procedural map generation using Perlin Noise :D
    use noise::{NoiseFn, Perlin};

    let perlin = Perlin::new(rand::random::<u32>()); // seed = 0

    let mut map = vec![vec![Tile::Water; height]; width]; // Inicia mapa lleno de agua

    for x in 0..width {
        for y in 0..height {
            let nx = x as f64 / width as f64;
            let ny = y as f64 / height as f64; 
            let gradient = ny; // Gradiente: 0.0 arriba, 1.0 abajo 
            let val = perlin.get([nx * 3.0, ny * 3.0]) + (gradient * 2.0 - 1.0);// este número controla el detalle
                                                                                // > detalle, islas + grandes

            //                   0.4 da más agua :D
            //                   0.0 es        50     &         50
            map[x][y] = if val > 0.4 { Tile::Land } else { Tile::Water };
        }
    }

    map[1][height - 2] = Tile::Base;
    Map { tiles: map }
}

/*
fn simulate(mut drone: Drone, map: &Vec<Vec<Tile>>) {
    let height = map.len();
    let width = map[0].len();

    for y in 0..height {
        for x in 0..width {
            if map[y][x] == Tile::Water {
                drone.move_to(x, y);
            }
        }
    }

    // volver a la base
    drone.move_to(0, 0);
}
*/
/*
pub fn autonomous_flight(drone: &mut Drone, targets: &Targets) {

    /*
    for target in targets.positions.iter() {
        println!("🚁 Saliendo hacia cuadrante {:?}", target);

        // ir al cuadrante
        drone.go_to(target);

        println!("📊 Muestra recolectada en {:?}", target);

        // volver a la base
        drone.go_to_base();

        println!("✅ Entregó muestra en la base");
    }

    println!("🎉 Misión completada");
    */
}
*/
fn evaluate_quadrant(map: &Map, qx: usize, qy: usize) -> Tile {
    let mut land_count = 0;
    let mut total = 0;

    for x in (qx * QUADRANT_SIZE as usize)..((qx+1) * QUADRANT_SIZE as usize) {
        for y in (qy * QUADRANT_SIZE as usize)..((qy+1) * QUADRANT_SIZE as usize) {
            total += 1;
            if map.tiles[x][y] == Tile::Land {
                land_count += 1;
            }
        }
    }

    let land_ratio = land_count as f32 / total as f32;
    if land_ratio > 0.05 { // cuanta tierra tiene el cuadrante
        Tile::Land
    } else {
        Tile::Water
    }
}

pub fn get_tile_type(map: &Map, x: usize, y: usize) -> Tile {
    return map.tiles[x][y]
}
