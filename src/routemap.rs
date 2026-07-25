use bevy::prelude::*;
use hexx::Hex;
use hexx::HexLayout;

#[derive(Resource)]
pub struct HexSettings {
    pub layout: HexLayout,
}

impl HexSettings {
    // Convert hex axial coordinates to world position (pointy-top orientation)
    pub fn hex_to_world_pos(&self, hex: Hex) -> hexx::Vec2 {
        let wp = self.layout.hex_to_world_pos(hex);
        info!("WorldPos of {:?} is {},{}", hex, wp.x, wp.y);
        wp
    }

    // Convert world position to hex axial coordinates (pointy-top orientation)
    pub fn world_pos_to_hex(&self, pos: hexx::Vec2) -> Hex {
        let hex = self.layout.world_pos_to_hex(pos);

info!("location of worldPos {},{} is {:?}", pos.x,pos.y,hex);
        hex
    }
}

#[derive(Component)]
pub struct HexTile {
    pub coord: Hex,
    pub tile_name: String,
}
