use openmind_conductor::*;

mod ensemble_tests {
    use super::*;
    use memory::MuscleMemory;
    use ensemble::{Ensemble, ConductIntent};

    #[test]
    fn test_add_agents() {
        let mut ens = Ensemble::new();
        ens.add_agent("sensor", MuscleMemory::new());
        ens.add_agent("motor", MuscleMemory::new());
        assert!(ens.get_agent("sensor").is_some());
        assert!(ens.get_agent("motor").is_some());
        assert!(ens.get_agent("light").is_none());
    }

    #[test]
    fn test_conduct_simple_task() {
        let mut ens = Ensemble::new();
        let mut mem = MuscleMemory::new();
        mem.add_pattern("read_temperature", "temp_sensor", vec![trit::Trit::PlusOne], 0.9);
        ens.add_agent("sensor", mem);

        let results = ens.conduct(&ConductIntent::new("read_temperature", 22.0)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "sensor");
        assert!((results[0].1 - 22.0 * 0.9).abs() < 0.01);
    }

    #[test]
    fn test_conduct_no_matching_pattern() {
        let mut ens = Ensemble::new();
        ens.add_agent("sensor", MuscleMemory::new());
        let results = ens.conduct(&ConductIntent::new("nonexistent", 10.0)).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_conduct_parallel_task() {
        let mut ens = Ensemble::new();
        let mut mem1 = MuscleMemory::new();
        mem1.add_pattern("act", "chord", vec![trit::Trit::PlusOne], 1.0);
        let mut mem2 = MuscleMemory::new();
        mem2.add_pattern("act", "chord", vec![trit::Trit::Zero], 0.8);

        ens.add_agent("a", mem1);
        ens.add_agent("b", mem2);

        let score = score::Score::builder()
            .parallel(vec![
                score::Step::new("a", "act", vec![]),
                score::Step::new("b", "act", vec![]),
            ])
            .build();

        let results = ens.execute_score_direct(&score).unwrap();
        assert_eq!(results.len(), 2);
        // Both agents have pattern "act" with different confidences
        assert!(results.iter().any(|(id, _, _)| id == "a"));
        assert!(results.iter().any(|(id, _, _)| id == "b"));
    }
}

mod score_tests {
    use super::*;
    use memory::MuscleMemory;

    #[test]
    fn test_build_score() {
        let score = score::Score::builder()
            .step("sensor", "read", vec![], measure::Timing::Immediate)
            .step("motor", "move", vec!["fast"], measure::Timing::After(100))
            .build();

        assert_eq!(score.instructions.len(), 2);
    }

    #[test]
    fn test_score_serialize_deserialize() {
        let score = score::Score::builder()
            .step("sensor", "read", vec![], measure::Timing::Immediate)
            .build();

        let json = score.to_json().unwrap();
        let restored = score::Score::from_json(&json).unwrap();
        assert_eq!(restored.instructions.len(), 1);
    }

    #[test]
    fn test_branching_true_path() {
        let mut ens = ensemble::Ensemble::new();
        let mut mem = MuscleMemory::new();
        mem.add_pattern("read_temp", "temp", vec![trit::Trit::PlusOne], 1.0);
        ens.add_agent("sensor", mem);
        ens.agents.get_mut("sensor").unwrap().last_value = Some(30.0);

        let score = score::Score::builder()
            .branch(
                score::Condition::Gt(score::Reading::Last("sensor".into()), 25.0),
                vec![score::Step::new("sensor", "hot", vec![])],
                vec![score::Step::new("sensor", "cold", vec![])],
            )
            .build();

        let results = ens.execute_score_direct(&score).unwrap();
        // True path: should execute "hot" step
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "hot");
    }

    #[test]
    fn test_branching_false_path() {
        let mut ens = ensemble::Ensemble::new();
        let mem = MuscleMemory::new();
        ens.add_agent("sensor", mem);
        ens.agents.get_mut("sensor").unwrap().last_value = Some(20.0);

        let score = score::Score::builder()
            .branch(
                score::Condition::Gt(score::Reading::Last("sensor".into()), 25.0),
                vec![score::Step::new("sensor", "hot", vec![])],
                vec![score::Step::new("sensor", "cold", vec![])],
            )
            .build();

        let results = ens.execute_score_direct(&score).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "cold");
    }

    #[test]
    fn test_parallel_steps_in_score() {
        let score = score::Score::builder()
            .parallel(vec![
                score::Step::new("a", "x", vec![]),
                score::Step::new("b", "y", vec![]),
                score::Step::new("c", "z", vec![]),
            ])
            .build();

        match &score.instructions[0] {
            score::Instruction::Parallel(steps) => assert_eq!(steps.len(), 3),
            _ => panic!("Expected Parallel"),
        }
    }
}

mod baton_tests {
    use super::*;

    #[tokio::test]
    async fn test_local_send_receive() {
        let mut baton = baton::LocalBaton::new();
        let mut rx = baton.register("test_agent");
        let resp_tx = baton.response_sender();

        // Spawn a handler
        let handle = tokio::spawn(async move {
            if let Some(_req) = rx.recv().await {
                resp_tx.send(baton::FlexResponse {
                    agent_id: "test_agent".into(),
                    value: 42.0,
                    success: true,
                }).await.unwrap();
            }
        });

        baton.send("test_agent", baton::FlexRequest {
            chord: "test".into(),
            args: vec![],
        }).await.unwrap();

        let resp = baton.recv().await.unwrap();
        assert_eq!(resp.value, 42.0);
        assert!(resp.success);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_baton() {
        let mock = baton::MockBaton::new();
        mock.send("agent1", baton::FlexRequest {
            chord: "act".into(),
            args: vec!["fast".into()],
        }).await.unwrap();

        let sent = mock.get_sent().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "agent1");
    }
}

mod measure_tests {
    use super::*;

    #[test]
    fn test_bpm_beat_duration() {
        let bpm = measure::Bpm(120.0);
        let dur = bpm.beat_duration();
        assert!((dur.as_secs_f64() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_bpm_60() {
        let bpm = measure::Bpm(60.0);
        assert!((bpm.beat_duration().as_secs_f64() - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_fermata_duration() {
        let measure = measure::Measure::new(measure::Bpm(120.0));
        let result = measure.fermata(
            measure::Fermata::ForDuration(50),
            |_| None,
        ).await;
        assert!(result);
    }
}

mod harmony_tests {
    use super::*;

    #[test]
    fn test_majority_vote() {
        let mut harmony = harmony::Harmony::new(harmony::VotingStrategy::Majority);
        let mut votes = std::collections::HashMap::new();
        votes.insert("a".into(), trit::Trit::PlusOne);
        votes.insert("b".into(), trit::Trit::PlusOne);
        votes.insert("c".into(), trit::Trit::MinusOne);

        let result = harmony.vote(votes);
        assert_eq!(result.outcome, trit::Trit::PlusOne);
        assert_eq!(result.for_count, 2);
        assert_eq!(result.against_count, 1);
    }

    #[test]
    fn test_consensus_blocked() {
        let mut harmony = harmony::Harmony::new(harmony::VotingStrategy::Consensus);
        let mut votes = std::collections::HashMap::new();
        votes.insert("a".into(), trit::Trit::PlusOne);
        votes.insert("b".into(), trit::Trit::MinusOne);

        let result = harmony.vote(votes);
        assert_eq!(result.outcome, trit::Trit::MinusOne); // blocked
    }

    #[test]
    fn test_consensus_unanimous() {
        let mut harmony = harmony::Harmony::new(harmony::VotingStrategy::Consensus);
        let mut votes = std::collections::HashMap::new();
        votes.insert("a".into(), trit::Trit::PlusOne);
        votes.insert("b".into(), trit::Trit::PlusOne);

        let result = harmony.vote(votes);
        assert_eq!(result.outcome, trit::Trit::PlusOne);
    }

    #[test]
    fn test_weighted_vote() {
        let mut weights = std::collections::HashMap::new();
        weights.insert("a".into(), 3.0);
        weights.insert("b".into(), 1.0);

        let mut harmony = harmony::Harmony::new(harmony::VotingStrategy::Weighted(weights));
        let mut votes = std::collections::HashMap::new();
        votes.insert("a".into(), trit::Trit::MinusOne);
        votes.insert("b".into(), trit::Trit::PlusOne);

        let result = harmony.vote(votes);
        // a=-1*3=-3, b=+1*1=1 => total=-2 => MinusOne
        assert_eq!(result.outcome, trit::Trit::MinusOne);
    }

    #[test]
    fn test_dissent_tracking() {
        let mut harmony = harmony::Harmony::new(harmony::VotingStrategy::Majority);

        let mut votes1 = std::collections::HashMap::new();
        votes1.insert("a".into(), trit::Trit::MinusOne);
        votes1.insert("b".into(), trit::Trit::PlusOne);
        votes1.insert("c".into(), trit::Trit::PlusOne);
        harmony.vote(votes1);
        assert_eq!(harmony.dissent_count("a"), 1);

        let mut votes2 = std::collections::HashMap::new();
        votes2.insert("a".into(), trit::Trit::MinusOne);
        votes2.insert("b".into(), trit::Trit::PlusOne);
        votes2.insert("c".into(), trit::Trit::PlusOne);
        harmony.vote(votes2);
        assert_eq!(harmony.dissent_count("a"), 2);
        assert_eq!(harmony.total_dissent(), 2);
    }
}

mod conductor_integration {
    use super::*;

    #[test]
    fn test_full_orchestration() {
        // 3 agents: sensor, motor, light
        let mut ens = ensemble::Ensemble::new();

        let mut sensor_mem = memory::MuscleMemory::new();
        sensor_mem.add_pattern("read_temperature", "temp", vec![trit::Trit::PlusOne], 1.0);
        ens.add_agent("sensor", sensor_mem);
        ens.agents.get_mut("sensor").unwrap().last_value = Some(30.0);

        let mut motor_mem = memory::MuscleMemory::new();
        motor_mem.add_pattern("fan_on", "motor", vec![trit::Trit::PlusOne], 1.0);
        motor_mem.add_pattern("fan_off", "motor", vec![trit::Trit::Zero], 1.0);
        ens.add_agent("motor", motor_mem);

        let mut light_mem = memory::MuscleMemory::new();
        light_mem.add_pattern("set_color", "led", vec![trit::Trit::PlusOne], 0.9);
        ens.add_agent("light", light_mem);

        // Complex score: read temp → branch (hot → fan on) → set light
        let score = score::Score::builder()
            .step("sensor", "read_temperature", vec![], measure::Timing::Immediate)
            .branch(
                score::Condition::Gt(score::Reading::Last("sensor".into()), 25.0),
                vec![score::Step::new("motor", "fan_on", vec![])],
                vec![score::Step::new("motor", "fan_off", vec![])],
            )
            .step("light", "set_color", vec![], measure::Timing::After(100))
            .build();

        let results = ens.execute_score_direct(&score).unwrap();
        // 3 steps executed: sensor read, motor fan_on (branch true), light set_color
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "sensor");
        assert_eq!(results[1].0, "motor");
        assert_eq!(results[1].1, "fan_on"); // branch took true path
        assert_eq!(results[2].0, "light");
    }
}
