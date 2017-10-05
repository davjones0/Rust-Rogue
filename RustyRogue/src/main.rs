extern crate tcod;

use tcod::console::*;
use tcod::colors::{self, Color};
use tcod::map::{Map as FovMap, FovAlgorithm};

// actual size of the window
const SCREEN_WIDTH: i32 = 90;
const SCREEN_HEIGHT: i32 = 60;

// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;
const MAP_DEPTH: i32 = 20;

const LIMIT_FPS: i32 = 20;

const COLOR_DARK_WALL: Color = Color { r: 100, g: 0, b: 0 };
const COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b:50 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50 };

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;
const PLAYER: usize = 0;

type Map = Vec<Tile>;

// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile{blocked: false, block_sight: false}
    }

    pub fn wall() -> Self {
        Tile{blocked: true, block_sight: true}
    }
}

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    z: i32,
    name: String,
    char: char,
    color: Color,
    blocks: bool,
    alive: bool,
    standing: bool
}

impl Object {
    pub fn new(x: i32, y: i32, z: i32, name: &str, char: char, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            z: z,
            name: name.into(),
            char: char,
            color: color,
            blocks: blocks,
            alive: false,
            standing: true
        }
    }

    /// dwarf fortress like standing/lying down
    pub fn standing(&mut self) {
        if self.standing == false {
            self.blocks = true;
            self.standing = true;
        } else {
            self.blocks = false;
            self.standing = false;
        }
    }

    pub fn pos(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn set_pos(&mut self, x: i32, y: i32, z: i32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    /// move by the given amount, if the destination is not blocked
    pub fn move_by(&mut self, dx: i32, dy: i32, dz: i32, map: &Map) {
        if !map[((self.x + dx) + MAP_WIDTH * ((self.y + dy) + MAP_DEPTH * (self.z + dz))) as usize].blocked {
            self.x += dx;
            self.y += dy;
            self.z += dz;
        }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut Console) {
        // let ax = self.x * MAP_WIDTH * (self.y + MAP_DEPTH * self.z)
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    /// Erase the character that represents this object
    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }
}


fn make_map() -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![Tile::empty(); (MAP_DEPTH * MAP_WIDTH * MAP_HEIGHT) as usize];
    // place two pillars to test the map
    map[(30 + MAP_WIDTH * (22 + MAP_DEPTH * 10)) as usize] = Tile::wall();
    //map[(30 + MAP_WIDTH * (22 + MAP_DEPTH * 11)) as usize] = Tile::wall();
    //map[(SCREEN_WIDTH / 2 * MAP_WIDTH + SCREEN_HEIGHT /2) as usize] = Tile::wall();
    map[(10 + MAP_WIDTH * (22 + MAP_DEPTH * 10)) as usize] = Tile::wall();
    //map[(50 + MAP_WIDTH * (22 + MAP_DEPTH * 11)) as usize] = Tile::wall();
    map[(10 + MAP_WIDTH * (12 + MAP_DEPTH * 1)) as usize] = Tile::wall();
    map[(15 + MAP_WIDTH * (15 + MAP_DEPTH * 0)) as usize] = Tile::empty();
    map[(0 + MAP_WIDTH * (2 + MAP_DEPTH * 0)) as usize] = Tile::wall();
    map[(30 + MAP_WIDTH * (29 + MAP_DEPTH * 1)) as usize] = Tile::wall();
    map[(23 + MAP_WIDTH * (6 + MAP_DEPTH * 0)) as usize] = Tile::wall();
    map[0] = Tile::wall();
    map[(6 + MAP_WIDTH * (6 + MAP_DEPTH * 1)) as usize] = Tile::wall();
    map[(30 + MAP_WIDTH * (25 + MAP_DEPTH * 0)) as usize] = Tile::wall();
    //println!("{:?}", map);
    map
}

fn is_blocked(x: i32, y: i32, z: i32, map: &Map, objects: &[Object]) -> bool {
    // first test the map tile
    if map[mAlg(x, y, z)].blocked {
        return true;
    }
    // now check for any blocking objects
    objects.iter().any(|object| {
        object.blocks && object.pos() == (x, y, z)
    })
}

fn render_all(root: &mut Root, con: &mut Offscreen, objects: &[Object], map: &Map, fov_map: &mut FovMap, fov_recompute: bool) {
    // go through all tiles, and set their background color
    //for z in 0..MAP_DEPTH {
    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[0];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let eq = x + MAP_WIDTH * (y + MAP_DEPTH * 10);
                let visible = fov_map.is_in_fov(x, y);
                let wall = map[eq as usize].block_sight;
                let color = match (visible, wall) {
                    // outside of field of view:
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    // inside fov:
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };
                con.set_char_background(x, y, color, BackgroundFlag::Set);
            }
        }
    //}

    // draw all objects in the list
    for object in objects {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }
    }

    // blit the contents of "con" to the root console
    blit(con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), root, (0, 0), 1.0, 1.0);
}

fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = root.wait_for_keypress(true);
    match key {
        Key { code: Enter, alt: true, ..} => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, ..} => return true, // exit game

        // movement keys
        Key { code: Up, .. } => player.move_by(0, -1, 0, map),
        Key { code: Down, .. } => player.move_by(0, 1, 0, map),
        Key { code: Left, .. } => player.move_by(-1, 0, 0, map),
        Key { code: Right, .. } => player.move_by(1, 0, 0, map),
        Key { printable: 's', .. } => player.standing(),

        _ => {},
    }

    false
}

fn mAlg(x: i32, y: i32, z:i32) -> usize {
    let eq = x + MAP_WIDTH * (y + MAP_DEPTH * z);
    return eq as usize;
}

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    tcod::system::set_fps(LIMIT_FPS);
    let mut con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    // create object representing the player
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, 10, "Player", '@', colors::WHITE, true);
    let mut previous_player_position = (-1, -1);
    // create an NPC
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, 10, "Orc", '@', colors::YELLOW, true);

    // the list of objects with those two
    let mut objects = [player, npc];

    // generate map (at this point it's not drawn to the screen)
    let map = make_map();

    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(x, y, !map[mAlg(x, y, 10)].block_sight, !map[mAlg(x, y, 10)].blocked);
        }
    }

    while !root.window_closed() {
        // render the screen
        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(&mut root, &mut con, &objects, &map, &mut fov_map, fov_recompute);

        root.flush();

        // erase all objects at their old locations, before they move
        for object in &objects {
            object.clear(&mut con)
        }

        // handle keys and exit game if needed
        let player = &mut objects[PLAYER];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut root, player, &map);
        if exit {
            break
        }
    }
}
