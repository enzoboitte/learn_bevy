use bevy::{ecs::{entity::Entity, resource::Resource}, math::Vec2, platform::collections::HashMap};

use crate::world::map::world_to_tile;


#[derive(Resource, Debug, Default)]
pub struct EntitiesWorld 
{
    entities: HashMap<(usize, usize), Entity>,
}

#[allow(dead_code)]
impl EntitiesWorld 
{
    pub fn insert(&mut self, position: Vec2, entity: Entity)
    {
        if let Some(pos) = world_to_tile(position)
        {
            self.entities.insert(pos, entity);
        }
    }

    pub fn get(&self, position: Vec2) -> Option<&Entity>
    {
        if let Some(pos) = world_to_tile(position)
        {
            self.entities.get(&pos)
        } else {
            None
        }
    }

    pub fn remove(&mut self, position: Vec2) -> Option<Entity> 
    {
        if let Some(pos) = world_to_tile(position)
        {
            self.entities.remove(&pos)
        } else {
            None
        }
    }

    pub fn remove_by_id(&mut self, entity: Entity) 
    {
        let key_to_remove = self.entities.iter()
            .find_map(|(key, &value)| if value == entity { Some(*key) } else { None });

        if let Some(key) = key_to_remove 
        {
            self.entities.remove(&key);
        }
    }

    pub fn is_tile_occupied(&self, position: Vec2) -> bool
    {
        if let Some(pos) = world_to_tile(position)
        {
            self.entities.contains_key(&pos)
        } else {
            false
        }
    }
}