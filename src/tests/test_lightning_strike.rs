#[cfg(test)]

pub mod test {
    use crate::{condition::Condition, test_environment::test::TestEnvironment};
    use common::{
        card::Card,
        entity_blueprint::EntityBlueprint,
        play_target::{PlayTarget, WorldPosTarget},
    };
    use macroquad::math::Vec2;

    #[test]
    fn test_lightning_strike() {
        let mut test_env = TestEnvironment::default();
        
        test_env.play_entity(test_env.player_a, EntityBlueprint::StreetCriminal.create());
        
        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        assert!(test_env.simulate_for(0.5).is_ok());
        let criminal_b =
            test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        assert!(test_env.simulate_for(0.5).is_ok());
        test_env.play_entity(test_env.player_b, EntityBlueprint::StreetCriminal.create());
        assert!(test_env.simulate_for(5.0).is_ok());

        let Vec2 { x, y } = test_env.get_entity_position(criminal_b);
        for _ in 0..5 {
            test_env.play_card_at(
                test_env.player_a,
                Card::LightningStrike,
                Some(PlayTarget::WorldPos(WorldPosTarget { x, y })),
            );
            assert!(test_env.simulate_for(0.5).is_ok());
        }
        let simulation_result = test_env.simulate_until(Condition::PlayerWon(test_env.player_a));
        assert!(simulation_result.is_ok());
    }
}
