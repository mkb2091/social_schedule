use crate::schedule;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Email {
    username: String,
    host: String,
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:}@{:}", self.username, self.host)
    }
}

impl Email {
    pub fn parse_string(email_string: &str) -> Result<Option<Email>, ()> {
        if email_string.is_empty() {
            return Ok(None);
        }
        let mut username: String = String::new();
        let mut host: String = String::new();
        let mut found_at: bool = false;
        for c in email_string.chars() {
            if !found_at {
                if c == '@' {
                    found_at = true;
                } else {
                    username.push(c);
                }
            } else if c == '@' {
                return Err(());
            } else {
                host.push(c);
            }
        }
        if !found_at || username.is_empty() || host.is_empty() {
            return Err(());
        }
        Ok(Some(Email { username, host }))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Player {
    pub name: String,
    pub email: Option<Email>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Group {
    pub name: String,
    players: std::collections::HashSet<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Event {
    pub name: String,
    schedule: schedule::Schedule,
    pub players: Vec<u32>,
}

impl Group {
    pub fn get_players(&self) -> std::collections::hash_set::Iter<u32> {
        self.players.iter()
    }
}

impl Event {
    pub fn from(name: String, schedule: schedule::Schedule, players: Vec<u32>) -> Option<Event> {
        Some(Event {
            name,
            schedule,
            players,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Database {
    players: std::collections::HashMap<u32, Player>,
    groups: std::collections::HashMap<u32, Group>,
    events: std::collections::HashMap<u32, Event>,
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
            events: std::collections::HashMap::new(),
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
    pub fn add_player(&mut self, name: String, email: Option<Email>) {
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
            let group_ids: Vec<u32> = self.groups.iter().map(|(&id, _)| id).collect();
            for group_id in group_ids {
                self.remove_player_from_group(group_id, id);
            }
            self.dump();
            Some(player)
        } else {
            None
        }
    }

    pub fn change_player_name(&mut self, id: u32, new_name: String) {
        if let Some(player) = self.players.get_mut(&id) {
            player.name = new_name;
            self.dump();
        }
    }

    pub fn change_player_email(&mut self, id: u32, new_email: Option<Email>) {
        if let Some(player) = self.players.get_mut(&id) {
            player.email = new_email;
            self.dump();
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

    pub fn remove_group(&mut self, id: u32) -> Option<Group> {
        if let Some(group) = self.groups.remove(&id) {
            self.dump();
            Some(group)
        } else {
            None
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
    pub fn remove_player_from_group(&mut self, group_id: u32, player_id: u32) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            if group.players.remove(&player_id) {
                self.dump();
            };
        }
    }
    pub fn change_group_name(&mut self, group_id: u32, new_name: String) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            group.name = new_name;
            self.dump();
        }
    }
    pub fn add_event(
        &mut self,
        name: String,
        event_schedule: schedule::Schedule,
        players: Vec<u32>,
    ) {
        if let Some(event) = Event::from(name, event_schedule, players) {
            for id in (self.events.len() as u32)..std::u32::MAX {
                if !self.events.contains_key(&id) {
                    self.events.insert(id, event);
                    self.dump();
                    return;
                }
            }
        }
    }
    pub fn get_events(&self) -> Vec<(&u32, &Event)> {
        self.events.iter().collect()
    }
}
