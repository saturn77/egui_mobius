//! this is the code for manipulating projects, to be used by the Crux backend.
//!
//! the frontend code does not have direct access to the 'Project' struct and should not
//! have a direct dependency on this library.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use shared_types::MemberKey;

#[derive(Debug)]
pub struct Project {
    // all fields are specifically NOT public.
    
    name: String,
    description: String,
    members: HashMap<MemberKey, Member>,
}

impl Project {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }
}

impl Project {
    pub fn new(name: String, description: String) -> Project {
        Project { name, description, members: HashMap::new() }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description
    }

    pub fn add_member(&mut self, name: String) {
        let member = Member {
            name: name,
        };
        let mut hasher = DefaultHasher::new();
        member.hash(&mut hasher);
        let hash = hasher.finish();

        self.members.insert(hash, member);
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Member {
    name: String,
}

impl Member {
    pub fn set_name(&mut self, name: String) {
        self.name = name
    }
}