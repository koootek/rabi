use rabi::{
    FromRaw, IntoRaw,
    derive::{FromRaw, IntoRaw},
};

#[test]
fn test_move() {
    server();
}

#[derive(Debug, FromRaw, IntoRaw)]
struct World {
    players: Vec::<Player>,
}

#[derive(Debug, FromRaw, IntoRaw)]
struct Player {
    name: String,
    visible: bool,
    health: f32,
    position: Vec3,
    pets: Vec::<usize>,
    messages: Vec::<String>,
}

#[derive(Debug, FromRaw, IntoRaw)]
struct Vec3 {
    x: i64,
    y: i64,
    z: i64,
}

fn server() {
    let mut world = World {
        players: vec![],
    };
    world.players.push(Player {
        name: "abc".to_string(),
        visible: true,
        health: 20.0,
        position: Vec3 { x: 0, y: 0, z: 0 },
        pets: vec![1, 2],
        messages: ["abc", "def"].iter().map(|s| s.to_string()).collect(),
    });
    world.players.push(Player {
        name: "xyz".to_string(),
        visible: true,
        health: 15.0,
        position: Vec3 { x: 3, y: 7, z: 2 },
        pets: vec![],
        messages: ["jkl", "mno"].iter().map(|s| s.to_string()).collect(),
    });
    println!("server pre: {world:#?}");
    world = World::from_raw(client(world.into_raw()));
    println!("server post: {world:#?}");
    let player = &world.players[0];
    assert_eq!(player.name, "abcdef".to_string());
    assert_eq!(player.health, 10.5);
    assert!(!player.visible);
    assert_eq!(player.position.x, 10);
    assert_eq!(player.position.y, 1);
    assert_eq!(player.position.z, -5);
    assert_eq!(player.pets, vec![3, 4]);
    assert_eq!(player.messages, vec!["abc", "def", "ghi"]);
    let player = &world.players[1];
    assert_eq!(player.name, "xyz".to_string());
    assert_eq!(player.health, 15.0);
    assert!(player.visible);
    assert_eq!(player.position.x, 3);
    assert_eq!(player.position.y, 7);
    assert_eq!(player.position.z, 2);
    assert_eq!(player.pets, vec![]);
    assert_eq!(player.messages, vec!["jkl", "mno"]);
}

fn client(world: rabi::Raw<World>) -> rabi::Raw<World> {
    let mut world = World::from_raw(world);
    println!("client pre: {world:#?}");
    let player = &mut world.players[0];
    player.name.push_str("def");
    player.health = 10.5;
    player.visible = false;
    player.position.x += 10;
    player.position.y += 1;
    player.position.z -= 5;
    for pet in &mut player.pets {
        *pet += 2;
    }
    player.messages.push("ghi".to_string());
    println!("client post: {world:#?}");
    world.into_raw()
}
