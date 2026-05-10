use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StatusEffect {
    Vulnerable,
    Weak,
    Poison,
    Strength,
    Ritual,
    Dexterity,
    Entangle,
    Frail,
    SharpHide,
    ModeShiftProgress,
    ModeShiftCount,
    GuardianMode,
    // Power card states
    DemonForm,
    Barricade,
    FeelNoPain,
    DarkEmbrace,
    Juggernaut,
    Unmovable,
    Rupture,
    Berserk,
    Brutality,
    Combust,
    Evolve,
    FireBreathing,
    StrengthDown,
    DexterityDown,
    Shackled,
    StonePlating,
    Enrage,
    Metallicize,
    Stunned,
    Sleep,
    CurlUp,
    Panache,
    Regen,
    Thorns,
    Artifact,
    SadisticNature,
    Mayhem,
}

impl StatusEffect {
    fn ticks_at_end_of_turn(self) -> bool {
        matches!(self, StatusEffect::Vulnerable | StatusEffect::Weak | StatusEffect::Entangle | StatusEffect::Frail | StatusEffect::Stunned | StatusEffect::Sleep)
    }

    pub fn is_debuff(self) -> bool {
        matches!(self, StatusEffect::Vulnerable | StatusEffect::Weak | StatusEffect::Frail | StatusEffect::Poison | StatusEffect::Entangle | StatusEffect::Stunned | StatusEffect::Sleep)
    }
}

pub type StatusMap = IndexMap<StatusEffect, i32>;

pub fn get_stacks(statuses: &StatusMap, effect: StatusEffect) -> i32 {
    statuses.get(&effect).copied().unwrap_or(0)
}

pub fn resolve_block(base: i32, statuses: &StatusMap) -> i32 {
    let base = base + get_stacks(statuses, StatusEffect::Dexterity);
    let base = if statuses.contains_key(&StatusEffect::Frail) { base * 3 / 4 } else { base };
    base.max(0)
}

pub fn resolve_damage(base: i32, attacker: &StatusMap, defender: &StatusMap) -> i32 {
    resolve_damage_with_strength_multiplier(base, 1, attacker, defender)
}

pub fn resolve_damage_with_strength_multiplier(base: i32, str_multiplier: i32, attacker: &StatusMap, defender: &StatusMap) -> i32 {
    let dmg = base + get_stacks(attacker, StatusEffect::Strength) * str_multiplier;
    let dmg = if attacker.contains_key(&StatusEffect::Weak) { dmg * 3 / 4 } else { dmg };
    let dmg = if defender.contains_key(&StatusEffect::Vulnerable) { dmg * 3 / 2 } else { dmg };
    dmg.max(0)
}

pub fn tick_statuses(statuses: &mut StatusMap) {
    statuses.retain(|&status, stacks| {
        if !status.ticks_at_end_of_turn() {
            return true;
        }
        *stacks -= 1;
        *stacks > 0
    });
}

/// Ticks ritual: returns Strength gained (= ritual stacks). Ritual does not decrement.
pub fn tick_ritual(statuses: &mut StatusMap) -> i32 {
    let ritual = get_stacks(statuses, StatusEffect::Ritual);
    if ritual > 0 {
        *statuses.entry(StatusEffect::Strength).or_insert(0) += ritual;
    }
    ritual
}

/// Removes StrengthDown and Shackled at end of turn.
/// Returns net Strength modifier: positive = net gain (Shackled), negative = net loss (StrengthDown).
pub fn tick_strength_modifiers(statuses: &mut StatusMap) -> i32 {
    let down = statuses.remove(&StatusEffect::StrengthDown).unwrap_or(0);
    let shackled = statuses.remove(&StatusEffect::Shackled).unwrap_or(0);
    shackled - down
}

/// Removes DexterityDown at end of turn.
/// Returns net Dexterity modifier: negative = net loss (DexterityDown).
pub fn tick_dexterity_modifiers(statuses: &mut StatusMap) -> i32 {
    let down = statuses.remove(&StatusEffect::DexterityDown).unwrap_or(0);
    -down
}

/// Ticks regen: heals `stacks` HP (capped at max_hp), decrements stacks by 1, removes when 0.
/// Returns amount healed.
pub fn tick_regen(statuses: &mut StatusMap, hp: &mut i32, max_hp: i32) -> i32 {
    let stacks = get_stacks(statuses, StatusEffect::Regen);
    if stacks == 0 {
        return 0;
    }
    let healed = stacks.min(max_hp - *hp).max(0);
    *hp = (*hp + healed).min(max_hp);
    if stacks == 1 {
        statuses.remove(&StatusEffect::Regen);
    } else {
        *statuses.get_mut(&StatusEffect::Regen).unwrap() -= 1;
    }
    healed
}

/// Drains poison: returns damage dealt and decrements the stack.
/// Returns 0 if no poison. Caller applies the damage to HP.
pub fn drain_poison(statuses: &mut StatusMap) -> i32 {
    let damage = get_stacks(statuses, StatusEffect::Poison);
    if damage == 0 {
        return 0;
    }
    if damage == 1 {
        statuses.remove(&StatusEffect::Poison);
    } else {
        *statuses.get_mut(&StatusEffect::Poison).unwrap() -= 1;
    }
    damage
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> StatusMap { StatusMap::new() }

    fn map_with(effect: StatusEffect, stacks: i32) -> StatusMap {
        let mut m = StatusMap::new();
        m.insert(effect, stacks);
        m
    }

    // --- resolve_damage ---

    #[test]
    fn vulnerable_multiplies_damage_by_3_over_2() {
        let defender = map_with(StatusEffect::Vulnerable, 2);
        assert_eq!(resolve_damage(6, &empty(), &defender), 9); // 6 * 3/2
    }

    #[test]
    fn weak_multiplies_damage_by_3_over_4() {
        let attacker = map_with(StatusEffect::Weak, 2);
        assert_eq!(resolve_damage(8, &attacker, &empty()), 6); // 8 * 3/4
    }

    #[test]
    fn strength_adds_flat_bonus_to_damage() {
        let attacker = map_with(StatusEffect::Strength, 2);
        assert_eq!(resolve_damage(6, &attacker, &empty()), 8); // 6 + 2
    }

    #[test]
    fn resolve_block_base_with_no_dexterity() {
        assert_eq!(resolve_block(5, &empty()), 5);
    }

    #[test]
    fn dexterity_adds_to_block() {
        let statuses = map_with(StatusEffect::Dexterity, 2);
        assert_eq!(resolve_block(5, &statuses), 7);
    }

    #[test]
    fn resolve_block_cannot_go_negative() {
        let statuses = map_with(StatusEffect::Dexterity, -10);
        assert_eq!(resolve_block(5, &statuses), 0);
    }

    #[test]
    fn ritual_adds_strength_each_tick_without_decrementing() {
        let mut statuses = map_with(StatusEffect::Ritual, 3);
        assert_eq!(tick_ritual(&mut statuses), 3);
        assert_eq!(statuses[&StatusEffect::Strength], 3);
        assert_eq!(statuses[&StatusEffect::Ritual], 3); // ritual persists
        assert_eq!(tick_ritual(&mut statuses), 3);
        assert_eq!(statuses[&StatusEffect::Strength], 6);
    }

    #[test]
    fn ritual_tick_with_no_ritual_returns_zero() {
        let mut statuses = empty();
        assert_eq!(tick_ritual(&mut statuses), 0);
        assert!(statuses.is_empty());
    }

    #[test]
    fn strength_applies_before_vulnerable_multiplier() {
        let attacker = map_with(StatusEffect::Strength, 2);
        let defender = map_with(StatusEffect::Vulnerable, 2);
        assert_eq!(resolve_damage(6, &attacker, &defender), 12); // (6 + 2) * 3/2
    }

    #[test]
    fn frail_reduces_block_by_25_percent() {
        let statuses = map_with(StatusEffect::Frail, 1);
        assert_eq!(resolve_block(8, &statuses), 6); // 8 * 3/4 = 6
    }

    #[test]
    fn frail_ticks_at_end_of_turn() {
        let mut statuses = map_with(StatusEffect::Frail, 2);
        tick_statuses(&mut statuses);
        assert_eq!(get_stacks(&statuses, StatusEffect::Frail), 1);
    }

    #[test]
    fn frail_expires_when_stacks_reach_zero() {
        let mut statuses = map_with(StatusEffect::Frail, 1);
        tick_statuses(&mut statuses);
        assert!(!statuses.contains_key(&StatusEffect::Frail));
    }

    // --- tick_strength_modifiers ---

    #[test]
    fn strength_down_returns_negative_modifier() {
        let mut statuses = map_with(StatusEffect::StrengthDown, 2);
        assert_eq!(tick_strength_modifiers(&mut statuses), -2);
    }

    #[test]
    fn strength_down_clears_after_tick() {
        let mut statuses = map_with(StatusEffect::StrengthDown, 2);
        tick_strength_modifiers(&mut statuses);
        assert!(!statuses.contains_key(&StatusEffect::StrengthDown));
    }

    #[test]
    fn shackled_returns_positive_modifier() {
        let mut statuses = map_with(StatusEffect::Shackled, 3);
        assert_eq!(tick_strength_modifiers(&mut statuses), 3);
    }

    #[test]
    fn shackled_clears_after_tick() {
        let mut statuses = map_with(StatusEffect::Shackled, 3);
        tick_strength_modifiers(&mut statuses);
        assert!(!statuses.contains_key(&StatusEffect::Shackled));
    }

    #[test]
    fn both_statuses_returns_net_shackled_minus_down() {
        let mut statuses = StatusMap::new();
        statuses.insert(StatusEffect::StrengthDown, 2);
        statuses.insert(StatusEffect::Shackled, 5);
        assert_eq!(tick_strength_modifiers(&mut statuses), 3);
    }

    #[test]
    fn neither_status_returns_zero() {
        let mut statuses = empty();
        assert_eq!(tick_strength_modifiers(&mut statuses), 0);
    }

    #[test]
    fn enrage_does_not_tick_at_end_of_turn() {
        let mut statuses = map_with(StatusEffect::Enrage, 2);
        tick_statuses(&mut statuses);
        assert_eq!(get_stacks(&statuses, StatusEffect::Enrage), 2);
    }

    #[test]
    fn stunned_ticks_at_end_of_turn() {
        let mut statuses = map_with(StatusEffect::Stunned, 2);
        tick_statuses(&mut statuses);
        assert_eq!(get_stacks(&statuses, StatusEffect::Stunned), 1);
    }

    #[test]
    fn stunned_expires_when_stacks_reach_zero() {
        let mut statuses = map_with(StatusEffect::Stunned, 1);
        tick_statuses(&mut statuses);
        assert!(!statuses.contains_key(&StatusEffect::Stunned));
    }

    #[test]
    fn sleep_ticks_at_end_of_turn() {
        let mut statuses = map_with(StatusEffect::Sleep, 3);
        tick_statuses(&mut statuses);
        assert_eq!(get_stacks(&statuses, StatusEffect::Sleep), 2);
    }

    #[test]
    fn sleep_expires_when_stacks_reach_zero() {
        let mut statuses = map_with(StatusEffect::Sleep, 1);
        tick_statuses(&mut statuses);
        assert!(!statuses.contains_key(&StatusEffect::Sleep));
    }

    #[test]
    fn metallicize_does_not_tick_at_end_of_turn() {
        let mut statuses = map_with(StatusEffect::Metallicize, 8);
        tick_statuses(&mut statuses);
        assert_eq!(get_stacks(&statuses, StatusEffect::Metallicize), 8);
    }

    // --- tick_dexterity_modifiers ---

    #[test]
    fn dexterity_down_returns_negative_modifier() {
        let mut statuses = map_with(StatusEffect::DexterityDown, 3);
        assert_eq!(tick_dexterity_modifiers(&mut statuses), -3);
    }

    #[test]
    fn dexterity_down_clears_after_tick() {
        let mut statuses = map_with(StatusEffect::DexterityDown, 3);
        tick_dexterity_modifiers(&mut statuses);
        assert!(!statuses.contains_key(&StatusEffect::DexterityDown));
    }

    #[test]
    fn no_dexterity_down_returns_zero() {
        let mut statuses = empty();
        assert_eq!(tick_dexterity_modifiers(&mut statuses), 0);
    }
}
