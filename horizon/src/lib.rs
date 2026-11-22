//! Durability Viewer for Horizon: renders equipped armor/tool icons and remaining durability on the HUD.

use mod_api::{self, ClientHost, Hooks, RenderGuiPostEvent};

const GREEN: u32 = 0xFF55_FF55;
const WHITE: u32 = 0xFFFF_FFFF;
const YELLOW: u32 = 0xFFFF_FF55;
const RED: u32 = 0xFFFF_5555;

/// Returns the HUD text color for a remaining/max durability ratio.
fn durability_color(remaining: u32, max_damage: u32) -> u32 {
    if max_damage == 0 {
        return WHITE;
    }
    let pct = (remaining * 100) / max_damage;
    if pct >= 80 {
        GREEN
    } else if pct >= 40 {
        WHITE
    } else if pct >= 20 {
        YELLOW
    } else {
        RED
    }
}

#[derive(Default)]
struct DurabilityViewerMod;

#[mod_api::horizon_mod]
impl Hooks for DurabilityViewerMod {
    fn render_gui_post(&self, host: &dyn ClientHost, event: &mut RenderGuiPostEvent<'_>) {
        let Some(player) = host.local_player() else {
            return;
        };
        let Some(slots) = player.inventory_slots(host) else {
            return;
        };
        let selected = player.selected_slot(host).unwrap_or(0) as u32;

        // Order: helmet (39), chest (38), legs (37), boots (36), main hand (selected), offhand (40).
        let target_slots = [39_u32, 38, 37, 36, selected, 40];
        let mut entries = Vec::new();
        for slot_idx in target_slots {
            let Some((_, stack)) = slots.iter().find(|(idx, _)| *idx == slot_idx) else {
                continue;
            };
            let Some(max_damage) = host.max_damage(stack.item()) else {
                continue;
            };
            if max_damage == 0 {
                continue;
            }
            let damage = stack.damage();
            let remaining = max_damage.saturating_sub(damage);
            let label = format!("{remaining}");
            let color = durability_color(remaining, max_damage);
            entries.push((stack, label, color));
        }

        if entries.is_empty() {
            return;
        }

        let [screen_w, screen_h] = event.draw.size();
        let row_h = 18.0_f32;
        let mut y = (screen_h - (entries.len() as f32) * row_h - 4.0).max(4.0);

        for (stack, label, color) in entries {
            let text_w = event.draw.text_width(&label);
            let icon_x = (screen_w - 20.0).max(4.0);
            let text_x = (icon_x - text_w - 4.0).max(4.0);
            let _ = event.draw.item_stack(stack, [icon_x, y], 1.0);
            event.draw.text(&label, [text_x, y + 4.0], color, true);
            y += row_h;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GREEN, RED, WHITE, YELLOW, durability_color};

    #[test]
    fn maps_durability_percentages_to_expected_colors() {
        assert_eq!(durability_color(90, 100), GREEN);
        assert_eq!(durability_color(60, 100), WHITE);
        assert_eq!(durability_color(25, 100), YELLOW);
        assert_eq!(durability_color(10, 100), RED);
    }
}
