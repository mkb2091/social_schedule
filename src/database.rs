#[derive(serde::Serialize, serde::Deserialize, Clone)]

use validators::ValidatorOption;
use validators::email::Email;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Player {
    pub name: String,
    pub email: Email
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Group {
    pub name: String,
    players: std::collections::HashSet<u32>,
}

impl Group {
    pub fn get_players(&self) -> std::collections::hash_set::Iter<u32> {
        self.players.iter()
    }
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
                if storage.set_item("database", &string_form).is_err() {
                    log!("Failed to dump database to disk");
                }
            }
        }
    }
    pub fn add_player(&mut self, name: String, email: Email) 
        for id in (self.players.len() as u32)..std::u32::MAX {
            if !self.players.contains_key(&id) {
                self.players.insert(id, Player { name, email });
                self.dump();
                return;
            }
        }
    }

    pub fn get_players(&self) -> Vec<(&u32, &Player)> {
        self.players.iter().collect()
    }

    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn contains_player(&self, id: u32) -> bool {
        self.players.contains_key(&id)
    }

    pub fn remove_player(&mut self, id: u32) -> Option<Player> {
        if let Some(player) = self.players.remove(&id) {
            self.dump();
            Some(player)
        } else {
            None
        }
    }

    pub fn add_group(&mut self, name: String) {
        for id in (self.groups.len() as u32)..std::u32::MAX {
            if !self.groups.contains_key(&id) {
                self.groups.insert(
                    id,
                    Group {
                        name,
                        players: std::collections::HashSet::new(),
                    },
                );
                self.dump();
                return;
            }
        }
    }

    pub fn get_groups(&self) -> Vec<(&u32, &Group)> {
        self.groups.iter().collect()
    }

    pub fn get_group(&self, id: u32) -> Option<&Group> {
        self.groups.get(&id)
    }

    pub fn add_player_to_group(&mut self, group_id: u32, player_id: u32) {
        if self.players.contains_key(&player_id) {
            if let Some(group) = self.groups.get_mut(&group_id) {
                group.players.insert(player_id);
                self.dump();
            }
        }
    }
}
