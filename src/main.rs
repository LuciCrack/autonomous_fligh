fn main() {
    let width = 60;
    let height = 60;

    // generar mapa procedural (puedes usar generate_coast_map con ruido)
    let map = generate_coast_map(width, height);

    // fase 1: reconocimiento
    let quadrants = reconnaissance(&map);
    println!("Reconocimiento completado, se mapearon {} cuadrantes", quadrants.len());

    // fase 2: guardar targets de agua
    let targets = water_targets(&quadrants);
    println!("Se encontraron {} cuadrantes de agua", targets.len());

    // fase 3: vuelo autónomo de ida y vuelta
    let drone = Drone::init();
    autonomous_flight(drone, targets);
}

// Constante que define el tamaño de cada cuadrante en metros
//const TILE_SIZE: u32 = 1; // meters
const QUADRANT_SIZE: u32 = 10; // meters
const BASE: Position = Position { x: 0, y: 0};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Water,
    Land,
    Base,
}

#[derive(Debug)]
struct Quadrant {
    qx: usize,
    qy: usize,
    kind: Tile,
}

fn reconnaissance(map: &Vec<Vec<Tile>>) -> Vec<Quadrant> {
    let width = map[0].len();
    let height = map.len();

    let quad_x = width / QUADRANT_SIZE as usize;
    let quad_y = height / QUADRANT_SIZE as usize;

    let mut quadrants = vec![];

    for y in 0..quad_y {
        for x in 0..quad_x {
            let kind = evaluate_quadrant(map, x, y);
            quadrants.push(Quadrant { qx: x, qy: y, kind });
        }
    }

    quadrants
}

fn water_targets(quadrants: &Vec<Quadrant>) -> Vec<Position> {
    let mut targets = vec![];

    for q in quadrants {
        if q.kind == Tile::Water {
            let center_x = q.qx * QUADRANT_SIZE as usize + QUADRANT_SIZE as usize / 2;
            let center_y = q.qy * QUADRANT_SIZE as usize + QUADRANT_SIZE as usize / 2;

            targets.push(Position { x: center_x, y: center_y });
        }
    }

    targets
}



#[derive(PartialEq, Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug)]
struct Drone {
    pos: Position,
    // battery: f32,
}

impl Drone {
    fn init() -> Drone {
        Drone {
            pos: Position { x: 0, y: 0 },
            // battery: 100.0,
        }
    }
    fn move_to(&mut self, x: usize, y: usize) {
        self.pos = Position { x, y };
        if self.pos == BASE {
            println!("Drone back to base at (0,0)");
        } else {     
            println!("Drone moved to: ({x}, {y})");
        }
    }

    fn move_towards(&mut self, target: Position) -> bool {
        if Self::in_range(&self.pos, &target) { // En un radio de 1 metro
            if self.pos == BASE {
                println!("Drone is back at base");
            }
            return true; // ya está en rango
        }

        if self.pos.x < target.x {
            self.pos.x += 1;
        } else if self.pos.x > target.x {
            self.pos.x -= 1;
        }
        if self.pos.y < target.y {
            self.pos.y += 1;
        } else if self.pos.y > target.y {
            self.pos.y -= 1;
        }

        false
    }

    fn go_to(&mut self, target: Position) {
        while !self.move_towards(target) {
            print!(".")
        }
        println!("\nDron llegó a {:?}", self.pos);
    }

    fn go_to_base(&mut self) {
        self.go_to(BASE);
    }

    fn in_range(pos: &Position, target: &Position) -> bool {
        return (pos.x as i32 - target.x as i32).abs() <= 1 &&
           (pos.y as i32 - target.y as i32).abs() <= 1;
    }
}



fn generate_coast_map(width: usize, height: usize) -> Vec<Vec<Tile>> {
    use noise::{NoiseFn, Perlin};

    let perlin = Perlin::new(rand::random::<u32>()); // seed = 0
    let mut map = vec![vec![Tile::Water; width]; height];

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let ny = y as f64 / height as f64;
            let val = perlin.get([nx * 5.0, ny * 5.0]); // 5.0 controla detalle

            map[y][x] = if val > 0.0 { Tile::Land } else { Tile::Water };
        }
    }

    map[0][0] = Tile::Base;
    map
}


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

fn autonomous_flight(mut drone: Drone, targets: Vec<Position>) {

    for target in targets {
        println!("🚁 Saliendo hacia cuadrante {:?}", target);

        // ir al cuadrante
        drone.go_to(target);

        println!("📊 Muestra recolectada en {:?}", target);

        // volver a la base
        drone.go_to_base();

        println!("✅ Entregó muestra en la base");
    }

    println!("🎉 Misión completada");
}

fn evaluate_quadrant(map: &Vec<Vec<Tile>>, qx: usize, qy: usize) -> Tile {
    let mut land_count = 0;
    let mut total = 0;

    for y in (qy * QUADRANT_SIZE as usize)..((qy+1) * QUADRANT_SIZE as usize) {
        for x in (qx * QUADRANT_SIZE as usize)..((qx+1) * QUADRANT_SIZE as usize) {
            total += 1;
            if map[y][x] == Tile::Land {
                land_count += 1;
            }
        }
    }

    let land_ratio = land_count as f32 / total as f32;
    if land_ratio > 0.1 { // más de 10% tierra
        Tile::Land
    } else {
        Tile::Water
    }
}



