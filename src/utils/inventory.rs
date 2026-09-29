use bevy::{ecs::resource::Resource, platform::collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Tomato,
    Wheat,
}

#[derive(Resource, Debug, Default)]
pub struct Inventory {
    items: HashMap<Item, u32>,
}

impl Inventory {
    pub fn add(&mut self, item: Item, quantity: u32)
    {
        *self.items.entry(item).or_insert(0) += quantity;
    }

    pub fn remove(&mut self, item: Item, quantity: u32) -> bool
    {
        if let Some(current_quantity) = self.items.get_mut(&item)
        {
            if *current_quantity >= quantity
            {
                *current_quantity -= quantity;
                return true;
            }
        }
        false
    }

    pub fn get_quantity(&self, item: Item) -> u32
    {
        *self.items.get(&item).unwrap_or(&0)
    }
}