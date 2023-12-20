use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

macro_rules! new_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub u64);

        impl $name {
            pub fn new() -> Self {
                Self(thread_rng().gen())
            }
        }
    };
}

new_id_type!(EntityId);
new_id_type!(CardInstanceId);
new_id_type!(BuildingLocationId);
new_id_type!(PathId);
new_id_type!(PlayerId);
new_id_type!(GameId);
