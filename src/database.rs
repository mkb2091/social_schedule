#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Player {
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Group {
    name: String,
    players: Vec<u32>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Database {
    players: std::collections::HashMap<u32, Player>,
    groups: std::collections::HashMap<u32, Group>,
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
            players: std::collections::HashMap::new(),
            groups: std::collections::HashMap::new(),
        }
    }

    pub fn dump(&self) {
        if let Ok(string_form) = serde_json::to_string(&self) {
            if let Some(storage) = seed::storage::get_storage() {
                if let Ok(_) = storage.set_item("database", &string_form) {
                }
            }
        }
    }
    pub fn add_player(&mut self, name: String) {
        for id in 0..std::u32::MAX {
            if !self.players.contains_key(&id) {
                self.players.insert(id, Player { name });
        		self.dump();
                return;
            }
        }
    }

    pub fn get_players(&self) -> Vec<(&u32, &Player)> {
        self.players.iter().collect()
    }
}
