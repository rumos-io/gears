use gaia_rs::abci_handler::GaiaABCIHandler;
use gaia_rs::config::AppConfig;
use gaia_rs::genesis::GenesisState;
use gaia_rs::store_keys::GaiaParamsStoreKey;
use gaia_rs::GaiaApplication;
use gears::baseapp::genesis::Genesis;
use gears::baseapp::options::NodeOptions;
use gears::baseapp::BaseApp;
use gears::config::Config;
use gears::store::database::MemDB;
use gears::tendermint::application::ABCIApplication;
use gears::tendermint::types::proto::block::BlockId;
use gears::tendermint::types::proto::consensus::{Consensus, ConsensusParams};
use gears::tendermint::types::proto::crypto::PublicKey;
use gears::tendermint::types::proto::header::{Header, PartSetHeader};
use gears::tendermint::types::proto::info::LastCommitInfo;
use gears::tendermint::types::proto::params::{BlockParams, EvidenceParams, ValidatorParams};
use gears::tendermint::types::proto::validator::ValidatorUpdate;
use gears::tendermint::types::request::begin_block::RequestBeginBlock;
use gears::tendermint::types::request::deliver_tx::RequestDeliverTx;
use gears::tendermint::types::request::end_block::RequestEndBlock;
use gears::tendermint::types::request::info::RequestInfo;
use gears::tendermint::types::request::init_chain::RequestInitChain;
use gears::tendermint::types::time::{Duration, Timestamp};

#[test]
/// In this scenario, we test the initialization of the application and submit a balance transfer on block three.
fn scenario_1() {
    let db = MemDB::new();
    let node_options = NodeOptions::new(
        "0uatom"
            .parse()
            .expect("hard coded min gas prices are valid"),
    );
    let config: Config<AppConfig> = Config::default();
    let app: BaseApp<MemDB, GaiaParamsStoreKey, GaiaABCIHandler, GaiaApplication> = BaseApp::new(
        db,
        GaiaParamsStoreKey::BaseApp,
        GaiaABCIHandler::new(config),
        node_options,
    );

    let mut genesis = GenesisState::default();
    genesis
        .add_genesis_account(
            "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux"
                .parse()
                .expect("hard coded address is valid"),
            "34uatom".parse().expect("hard coded coin is valid"),
        )
        .expect("won't fail since there's no existing account");

    let init_chain_request = RequestInitChain {
        time: Timestamp {
            seconds: 0,
            nanos: 0,
        },
        chain_id: "test-chain".parse().expect("hard coded chain id is valid"),
        consensus_params: ConsensusParams {
            block: BlockParams {
                max_bytes: 22020096,
                max_gas: -1,
            },
            evidence: EvidenceParams {
                max_age_num_blocks: 100000,
                max_age_duration: Some(Duration {
                    seconds: 172800,
                    nanos: 0,
                }),
                max_bytes: 1048576,
            },
            validator: ValidatorParams {
                pub_key_types: vec!["ed25519".into()],
            },
            version: None,
        },
        validators: vec![ValidatorUpdate {
            pub_key: PublicKey::Ed25519(vec![
                169, 51, 131, 24, 28, 140, 127, 3, 124, 161, 173, 82, 91, 30, 31, 191, 193, 162,
                209, 32, 251, 214, 246, 36, 207, 6, 120, 6, 156, 148, 139, 213,
            ]),
            power: 10,
        }],
        app_genesis: genesis,
        initial_height: 1,
    };
    app.init_chain(init_chain_request);

    let info_request = RequestInfo {
        version: "1".into(),
        block_version: 12,
        p2p_version: 12,
    };
    let _info_response = app.info(info_request);

    let request_begin_block = RequestBeginBlock {
            header: Header {
                version: Consensus { block: 11, app: 10 },
                chain_id: "test-chain".parse().expect("hard coded chain id is valid"),
                height: 1,
                time: Timestamp {
                    seconds: 0,
                    nanos: 0,
                },
                last_block_id: BlockId {
                    hash: vec![].into(),
                    part_set_header: Some(PartSetHeader {
                        total: 0,
                        hash: vec![].into(),
                    }),
                },
                last_commit_hash: vec![227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85].into(),
                data_hash: vec![227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85].into(),
                validators_hash: vec![105, 109, 157, 224, 221, 36, 139, 200, 18, 31, 171, 146, 191, 69, 50, 98, 210, 209, 111, 225, 255, 132, 34, 75, 183, 135, 230, 89, 52, 173, 104, 13].into(),
                next_validators_hash: vec![105, 109, 157, 224, 221, 36, 139, 200, 18, 31, 171, 146, 191, 69, 50, 98, 210, 209, 111, 225, 255, 132, 34, 75, 183, 135, 230, 89, 52, 173, 104, 13].into(),
                consensus_hash: vec![4, 128, 145, 188, 125, 220, 40, 63, 119, 191, 191, 145, 215, 60, 68, 218, 88, 195, 223, 138, 156, 188, 134, 116, 5, 216, 183, 243, 218, 173, 162, 47].into(),
                app_hash: vec![104, 97, 115, 104, 95, 103, 111, 101, 115, 95, 104, 101, 114, 101].into(),
                last_results_hash: vec![227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85].into(),
                evidence_hash: vec![227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200, 153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82, 184, 85].into(),
                proposer_address: vec![139, 66, 235, 161, 172, 24, 201, 229, 172, 156, 56, 187, 215, 206, 138, 87, 207, 173, 214, 85].into(),
            },
            last_commit_info: Some(LastCommitInfo {
                round: 0,
                votes: vec![],
            }),
            byzantine_validators: vec![],
            hash:  b"\xaaw\xbd^\x9d\x041\xfdc\x17\x11\x82\xb9iU\xde2\xd0\x19\xca\xdeV\x0e\x7fK\x1c\x88\xb6\xa3\xe3\x8b\x89".as_slice().into(),
        };
    app.begin_block(request_begin_block);

    let request_end_block = RequestEndBlock { height: 1 };
    app.end_block(request_end_block);

    let response = app.commit();
    let hash = hex::encode(response.data);

    assert_eq!(
        hash,
        "d0254da38fc9c97292f65f4e8af3276209c6d6f8a922bbad8fc4a8f36af55f67"
    );

    let request_deliver_tx = RequestDeliverTx { tx: vec![].into() };

    app.deliver_tx(request_deliver_tx);
}
