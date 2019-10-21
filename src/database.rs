#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Player {
    name: String,
    id: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Group {
    name: String,
    players: Vec<u32>,
    id: u32,
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Database {
    players: Vec<Player>,
    groups: Vec<Group>,
}

impl Database {
    pub fn load() -> Database {
        if let Some(storage) = seed::storage::get_storage() {
            if let Ok(Some(loaded_serialized)) = storage.get_item("database") {
                if let Ok(database) = serde_json::from_str(&loaded_serialized) {
                    return database;
                }
            }
        }
        Database {
            players: vec![],
            groups: vec![],
        }
    }

    pub fn dump(&self) {
        if let Ok(string_form) = serde_json::to_string(&self) {
            if let Some(storage) = seed::storage::get_storage() {
                if let Ok(_) = storage.set_item("database", &string_form) {}
            }
        }
    }
    pub fn add_player(&mut self, name: String) {
        for id in 0..std::u32::MAX {
            let mut exists: bool = false;
            for player in &self.players {
                if player.id == id {
                    exists = true;
                    break;
                }
            }
            if !exists {
                self.players.push(Player { name, id });
                return;
            }
        }
    }
}
