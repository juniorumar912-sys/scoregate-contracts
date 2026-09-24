use crate::{ScoreGateAggregator, ScoreGateAggregatorClient};
use scoregate_score::{ScoreGateScoreContract, ScoreGateScoreContractClient};
use scoregate_test_support::generate_score_roles;
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol, Vec};

fn setup_shard(env: &Env) -> (Address, ScoreGateScoreContractClient<'_>) {
    let shard = env.register_contract(None, ScoreGateScoreContract);
    let client = ScoreGateScoreContractClient::new(env, &shard);
    let (admin, service) = generate_score_roles(env);
    client.initialize(&admin, &service);
    (shard, client)
}

fn setup_aggregator(env: &Env) -> ScoreGateAggregatorClient<'_> {
    let aggregator = env.register_contract(None, ScoreGateAggregator);
    let client = ScoreGateAggregatorClient::new(env, &aggregator);
    client.initialize(&Address::generate(env));
    client
}

fn submit_score(
    env: &Env,
    shard: &ScoreGateScoreContractClient<'_>,
    wallet: &Address,
    pair: &Symbol,
    score: u32,
    timestamp: u64,
) {
    shard.submit_score(
        &Vec::new(env),
        wallet,
        pair,
        &score,
        &false,
        &false,
        &timestamp,
        &90,
        &1,
        &None,
    );
}

#[test]
fn get_score_arbitrates_real_shard_contracts() {
    let env = Env::default();
    env.mock_all_auths();
    let aggregator = setup_aggregator(&env);
    let (first_id, first) = setup_shard(&env);
    let (second_id, second) = setup_shard(&env);
    aggregator.add_shard(&first_id);
    aggregator.add_shard(&second_id);

    let wallet = Address::generate(&env);
    let pair = Symbol::new(&env, "XLM_USDC");
    submit_score(&env, &first, &wallet, &pair, 75, 1_000);
    submit_score(&env, &second, &wallet, &pair, 75, 1_000);

    let selected = aggregator.get_score(&wallet, &pair);
    assert_eq!(selected.score, 75);
    assert_eq!(selected.timestamp, 1_000);
}

#[test]
fn get_score_ignores_stale_high_score_from_real_shard() {
    let env = Env::default();
    env.mock_all_auths();
    let aggregator = setup_aggregator(&env);
    let (fresh_id, fresh) = setup_shard(&env);
    let (stale_id, stale) = setup_shard(&env);
    aggregator.add_shard(&fresh_id);
    aggregator.add_shard(&stale_id);

    let wallet = Address::generate(&env);
    let pair = Symbol::new(&env, "XLM_USDC");
    submit_score(&env, &fresh, &wallet, &pair, 25, 900_000);
    submit_score(&env, &stale, &wallet, &pair, 95, 1);
    env.ledger().with_mut(|ledger| ledger.timestamp = 1_000_000);

    let selected = aggregator.get_score(&wallet, &pair);
    assert_eq!(selected.score, 25);
    assert_eq!(selected.timestamp, 900_000);
}
