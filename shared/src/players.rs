use std::sync::{Arc, Mutex};

use vec_map::VecMap;

use crate::game::Game;

pub struct Players<G: Game> {
    players: Arc<Mutex<VecMap<G::Player>>>
}

impl<G: Game> Clone for Players<G> {
    fn clone(&self) -> Players<G> {
        Players {
            players: self.players.clone()
        }
    }
}

fn next_index<T>(map: &VecMap<T>) -> usize {
    let mut index = 0usize;

    for key in map.keys() {
        if index < key {
            return index;
        }

        index += 1
    }

    index
}

impl<G: Game> Players<G> {
    pub fn new() -> Players<G> {
        Players {
            players: Arc::new(Mutex::new(VecMap::new()))
        }
    }
    
    pub fn add_player(&mut self, player: G::Player) -> usize {
        let mut map = self.players.lock().unwrap();
        let index = next_index(&map);

        map.insert(index, player);
        index
    }

    pub fn update_player(&mut self, index: usize, player: G::Player) {
        self.players.lock().unwrap().insert(index, player);
    }

    pub fn with<F: Fn(usize, &G::Player)>(&self, f: F) {
        let map = self.players.lock().unwrap();
        let map_ref: &VecMap<G::Player> = &map;

        for (index,  player) in map_ref.into_iter() {
            f(index, player)
        }
    }

    pub fn player(&self, index: usize) -> Option<G::Player> {
        self.players.lock().unwrap().get(index).cloned()
    }
}