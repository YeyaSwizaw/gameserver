use server::game::Game;

pub struct TestGame;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Player {
    name: String
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            name
        }
    }
}

impl Game for TestGame {
    type Player = Player;
}
