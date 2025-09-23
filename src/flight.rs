use bevy::prelude::*;

// Constante que define el tamaño de cada cuadrante en metros
const QUADRANT_SIZE: f32 = 10.0; // meters
// Not used for now but might be usefull later :D
// const BASE: Vec2 = Vec2::new(0.0, 0.0);
const SPEED: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Water,
    Land,
    Base,
}

#[derive(Debug)]
pub struct Quadrant {
    qx: f32,
    qy: f32,
    kind: Tile,
    done: bool,
}

#[derive(Resource)]
pub struct Targets {
    pub positions: Vec<Vec2>,
}

pub fn reconnaissance(map: &Map) -> Vec<Quadrant> {
    let width = map.tiles[0].len();
    let height = map.tiles.len();

    let quad_x = width / QUADRANT_SIZE as usize;
    let quad_y = height / QUADRANT_SIZE as usize;

    let mut quadrants = vec![];

    for y in 0..quad_y {
        for x in 0..quad_x {
            let kind = evaluate_quadrant(map, x, y);
            quadrants.push(Quadrant { qx: x as f32, qy: y as f32, kind, done: false });
        }
    }

    quadrants
}

pub fn water_targets(quadrants: &Vec<Quadrant>) -> Targets {
    let mut targets = vec![];

    for q in quadrants {
        if q.kind == Tile::Water {
            let center_x = q.qx * QUADRANT_SIZE + QUADRANT_SIZE / 2.0;
            let center_y = q.qy * QUADRANT_SIZE + QUADRANT_SIZE / 2.0;

            targets.push(Vec2::new(center_x, center_y));
        }
    }

    Targets {
        positions: targets
    }
}


#[derive(Debug, Component)]
pub struct Drone {
    pub pos: Vec2,
    pub target: Option<Vec2>,
    pub speed: f32,
    // battery: f32,
}

impl Drone {
    pub fn init() -> Drone {
        Drone {
            pos: Vec2::new(0.0, 0.0),
            target: None,
            speed: SPEED, // px per second
            // battery: 100.0,
        }
    }
    pub fn init_with_pos(x: usize, y: usize) -> Drone {
        Drone {
            pos: Vec2::new(x as f32, y as f32),
            target: None,
            speed: SPEED,
        }
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
    use noise::{NoiseFn, Perlin};

    let perlin = Perlin::new(rand::random::<u32>()); // seed = 0
    let mut map = vec![vec![Tile::Water; width]; height];

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let ny = y as f64 / height as f64; 
            let gradient = nx; // Gradiente: 0.0 arriba, 1.0 abajo 
            let val = perlin.get([nx * 3.0, ny * 3.0]) + (gradient * 2.0 - 1.0);// este número controla el detalle
                                                                                // > detalle, islas + grandes

            //                   0.4 da más agua :D
            //                   0.0 es        50     &         50
            map[y][x] = if val > 0.4 { Tile::Land } else { Tile::Water };
        }
    }

    map[1][width - 2] = Tile::Base;
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

fn evaluate_quadrant(map: &Map, qx: usize, qy: usize) -> Tile {
    let mut land_count = 0;
    let mut total = 0;

    for y in (qy * QUADRANT_SIZE as usize)..((qy+1) * QUADRANT_SIZE as usize) {
        for x in (qx * QUADRANT_SIZE as usize)..((qx+1) * QUADRANT_SIZE as usize) {
            total += 1;
            if map.tiles[y][x] == Tile::Land {
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
