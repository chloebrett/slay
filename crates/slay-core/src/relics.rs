use crate::combat::Player;
use crate::types::Hp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relic {
    Strawberry,
    Pear,
    Mango,
}

pub fn grant_relic(player: &mut Player, relic: Relic) {
    match &relic {
        Relic::Strawberry => raise_max_hp(player, 7),
        Relic::Pear       => raise_max_hp(player, 10),
        Relic::Mango      => raise_max_hp(player, 14),
    }
    player.relics.push(relic);
}

fn raise_max_hp(player: &mut Player, amount: i32) {
    player.max_hp = Hp(player.max_hp.0 + amount);
    player.hp = Hp(player.hp.0 + amount);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;
    use crate::status::StatusMap;
    use crate::types::{Block, Energy};

    fn test_player() -> Player {
        Player {
            hp: Hp(80),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand: vec![],
            draw_pile: vec![],
            discard_pile: vec![],
            exhaust_pile: vec![],
            statuses: StatusMap::new(),
            deck: vec![],
            gold: 0,
            relics: vec![],
        }
    }

    #[test]
    fn strawberry_raises_max_hp_by_7() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Strawberry);
        assert_eq!(player.max_hp, Hp(87));
    }

    #[test]
    fn strawberry_raises_current_hp_by_7() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Strawberry);
        assert_eq!(player.hp, Hp(87));
    }

    #[test]
    fn strawberry_when_damaged_still_raises_hp_by_7() {
        let mut player = test_player();
        player.hp = Hp(50);
        grant_relic(&mut player, Relic::Strawberry);
        assert_eq!(player.hp, Hp(57));
        assert_eq!(player.max_hp, Hp(87));
    }

    #[test]
    fn pear_raises_max_hp_by_10() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Pear);
        assert_eq!(player.max_hp, Hp(90));
        assert_eq!(player.hp, Hp(90));
    }

    #[test]
    fn mango_raises_max_hp_by_14() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Mango);
        assert_eq!(player.max_hp, Hp(94));
        assert_eq!(player.hp, Hp(94));
    }

    #[test]
    fn strawberry_is_recorded_in_player_relics() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Strawberry);
        assert!(player.relics.contains(&Relic::Strawberry));
    }
}
