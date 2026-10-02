use bevy::prelude::*;
use lib_gui::prelude::*;

use crate::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item 
{
    Empty,
    Hoe,
    WateringCan,
    Axe,
    Pickaxe,
    FishingRod,
    Wheat,
    Tomato,
}

// Where an item's image is: which file, which pixel rectangle, and how much to enlarge it.
pub struct ItemSprite 
{
    pub file: &'static str,
    pub rect: Rect,
    pub scale: f32,
}

impl Item 
{
    pub fn name(self) -> &'static str 
    {
        match self 
        {
            Item::Empty => "",
            Item::Hoe => "Houe",
            Item::WateringCan => "Arrosoir",
            Item::Axe => "Hache",
            Item::Pickaxe => "Pioche",
            Item::FishingRod => "Canne a peche",
            Item::Wheat => "Ble",
            Item::Tomato => "Tomate",
        }
    }

    pub fn sprite(self) -> Option<ItemSprite> 
    {
        const UI: &str = "ui/ui.png";
        const PLANTS: &str = "tilesets/plants.png";

        // Tools: slots of ui.png (18x19, one every 32 px). Crops: 16x16 tiles of plants.png.
        let tool = |column: f32| (UI, Rect::new(487.0 + 32.0 * column, 7.0, 505.0 + 32.0 * column, 26.0), S);
        let crop = |row: f32| (PLANTS, Rect::new(80.0, 16.0 * row, 96.0, 16.0 * row + 16.0), 2.0);

        let (file, rect, scale) = match self 
        {
            Item::Empty => return None,
            Item::Hoe => tool(0.0),
            Item::WateringCan => tool(1.0),
            Item::Axe => tool(2.0),
            Item::Pickaxe => tool(3.0),
            Item::FishingRod => tool(4.0),
            Item::Wheat => crop(0.0),
            Item::Tomato => crop(1.0),
        };
        Some(ItemSprite { file, rect, scale })
    }
}

// Number of slots of the inventory (= slots shown in the hotbar).
pub const MAX_ITEM: usize = 6;
// Pixel scale of the UI (sprites are drawn 3x bigger).
const S: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot 
{
    pub item: Item,
    pub quantity: u32,
}

impl Slot 
{
    pub const EMPTY: Slot = Slot { item: Item::Empty, quantity: 0 };
}

#[derive(Resource, Debug)]
pub struct Inventory 
{
    pub slots: [Slot; MAX_ITEM],
    pub selected: usize,
}

impl Default for Inventory 
{
    fn default() -> Self 
    {
        let mut inventory = Self { slots: [Slot::EMPTY; MAX_ITEM], selected: 0 };
        for tool in [Item::Hoe, Item::WateringCan, Item::Axe, Item::Pickaxe, Item::FishingRod] 
        {
            inventory.add(tool, 1);
        }
        inventory.add(Item::Tomato, 1);
        inventory
    }
}

#[allow(dead_code)]
impl Inventory 
{
    
    pub fn add(&mut self, item: Item, quantity: u32) -> bool
    {
        let index = self.slots.iter().position(|s| s.item == item)
            .or_else(|| self.slots.iter().position(|s| s.item == Item::Empty));

        let Some(index) = index else { return false };
        self.slots[index].item = item;
        self.slots[index].quantity += quantity;
        true
    }

    pub fn remove(&mut self, item: Item, quantity: u32) -> bool
    {
        let Some(slot) = self.slots.iter_mut().find(|s| s.item == item && s.quantity >= quantity) else 
        {
            return false;
        };
        slot.quantity -= quantity;
        if slot.quantity == 0 
        {
            *slot = Slot::EMPTY;
        }
        true
    }

    pub fn get_quantity(&self, item: Item) -> u32
    {
        self.slots.iter().filter(|s| s.item == item).map(|s| s.quantity).sum()
    }

    pub fn is_full(&self, item: Item) -> bool
    {
        !self.slots.iter().any(|s| s.item == item || s.item == Item::Empty)
    }

    pub fn selected_item(&self) -> Item 
    {
        self.slots[self.selected].item
    }

    pub fn has_item(&self, item: Item) -> bool 
    {
        self.slots.iter().any(|s| s.item == item && s.quantity > 0)
    }
}

pub struct InventoryGuiPlugin;

impl Plugin for InventoryGuiPlugin
{
    fn build(&self, app: &mut App) 
    {
        app
            .add_systems(Update, (rebuild_hotbar, keyboard_select).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
struct HotbarUi;

// builds the hotbar whenever the content of the inventory slots changes.
fn rebuild_hotbar(
    mut commands: Commands,
    inventory: Res<Inventory>,
    assets: Res<AssetServer>,
    old: Query<Entity, With<HotbarUi>>,
    mut shown: Local<Option<[Slot; MAX_ITEM]>>,
) 
{
    if *shown == Some(inventory.slots) 
    {
        return;
    }
    *shown = Some(inventory.slots);
    for entity in &old 
    {
        commands.entity(entity).despawn();
    }

    let ui: Handle<Image> = assets.load("ui/ui.png");
    let slots = inventory.slots.iter().enumerate().map(|(i, content)| slot(&assets, &ui, i, *content));

    commands.spawn_view(
        vstack((
            spacer(),
            dyn_text(|inventory: &Inventory| inventory.selected_item().name().to_string())
                .headline()
                .text_shadow(Color::BLACK, 2.0, 2.0),
            hstack(slots.collect::<Vec<_>>()).spacing(6.0 * S),
        ))
        .spacing(12.0)
        .padding(24.0)
        .fill()
        .insert(HotbarUi),
    );
}

// Empty slot background + item image + quantity + selection corners.
fn slot(assets: &AssetServer, ui: &Handle<Image>, i: usize, content: Slot) -> impl View 
{
    let icon = content.item.sprite().map(|sprite| 
    {
        image(assets.load(sprite.file))
            .region(sprite.rect.min.x, sprite.rect.min.y, sprite.rect.width(), sprite.rect.height())
            .frame(sprite.rect.width() * sprite.scale, sprite.rect.height() * sprite.scale)
    });

    zstack((
        image(ui.clone()).region(455.0, 7.0, 18.0, 19.0).frame(18.0 * S, 19.0 * S),
        icon,
        (content.quantity > 1).then(|| 
        {
            text(content.quantity.to_string())
                .font_size(16.0)
                .text_shadow(Color::BLACK, 1.0, 1.0)
                .align_self(AlignSelf::End)
                .justify_self(JustifySelf::End)
                .margin(4.0)
        }),
        selection_corners(ui, i),
    ))
    .hover_scale(1.1)
    .press_scale(0.9)
    .tap_bounce()
    .on_tap(move |mut inventory: ResMut<Inventory>| inventory.selected = i)
}

fn selection_corners(ui: &Handle<Image>, i: usize) -> impl View 
{
    let corner = |(right, bottom): (bool, bool)| 
    {
        let outside = px(-3.0 * S);
        image(ui.clone())
            .region(if right { 404.0 } else { 388.0 }, if bottom { 20.0 } else { 4.0 }, 8.0, 9.0)
            .frame(8.0 * S, 9.0 * S)
            .node(move |n| 
            {
                n.position_type = PositionType::Absolute;
                if right { n.right = outside } else { n.left = outside }
                if bottom { n.bottom = outside } else { n.top = outside }
            })
    };

    zstack(for_each([(false, false), (true, false), (false, true), (true, true)], corner))
        .fill()
        .node(|n| n.position_type = PositionType::Absolute)
        .pulse(0.06, 1.2)
        .show_if(move |inventory: &Inventory| inventory.selected == i)
}

fn keyboard_select(keys: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) 
{
    let digits = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5,
                  KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9];
    if let Some(i) = digits.iter().take(MAX_ITEM).position(|key| keys.just_pressed(*key)) 
    {
        inventory.selected = i;
    }
}
