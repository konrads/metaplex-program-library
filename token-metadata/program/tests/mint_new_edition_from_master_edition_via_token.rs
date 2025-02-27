#![cfg(feature = "test-bpf")]
pub mod utils;

use borsh::BorshSerialize;
use mpl_token_metadata::{
    error::MetadataError,
    id, instruction,
    state::{
        Key, MasterEditionV2 as ProgramMasterEdition, TokenMetadataAccount, MAX_MASTER_EDITION_LEN,
    },
};
use num_traits::FromPrimitive;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use utils::*;

// NOTE: these tests depend on the token-vault program having been compiled
// via (cd ../../token-vault/program/ && cargo build-bpf)
mod mint_new_edition_from_master_edition_via_token {
    use super::*;
    use mpl_token_metadata::state::Collection;
    use solana_sdk::account::AccountSharedData;
    #[tokio::test]
    async fn success() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);

        test_metadata
            .create(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                0,
            )
            .await
            .unwrap();

        test_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        test_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = test_edition_marker.get_data(&mut context).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
    }

    #[tokio::test]
    async fn success_v2() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_collection = Metadata::new();
        test_collection
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let collection_master_edition_account = MasterEditionV2::new(&test_collection);
        collection_master_edition_account
            .create_v3(&mut context, Some(0))
            .await
            .unwrap();
        test_metadata
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                None,
            )
            .await
            .unwrap();

        test_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();
        let kpbytes = &context.payer;
        let kp = Keypair::from_bytes(&kpbytes.to_bytes()).unwrap();
        test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                &kp,
                test_collection.mint.pubkey(),
                collection_master_edition_account.pubkey,
                None,
            )
            .await
            .unwrap();
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);
        test_edition_marker.create(&mut context).await.unwrap();

        let edition_marker = test_edition_marker.get_data(&mut context).await;

        assert_eq!(edition_marker.ledger[0], 64);
        assert_eq!(edition_marker.key, Key::EditionMarker);
    }

    #[tokio::test]
    async fn fail_invalid_token_program() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);

        test_metadata
            .create(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                0,
            )
            .await
            .unwrap();

        test_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        let result = test_edition_marker
            .create_with_invalid_token_program(&mut context)
            .await
            .unwrap_err();
        assert_custom_error!(result, MetadataError::InvalidTokenProgram);
    }

    #[tokio::test]
    async fn fail_invalid_mint() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);
        let fake_mint = Keypair::new();
        let fake_account = Keypair::new();
        let payer_pubkey = context.payer.pubkey();

        test_metadata
            .create(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                0,
            )
            .await
            .unwrap();

        test_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        create_mint(&mut context, &fake_mint, &payer_pubkey, None, 0)
            .await
            .unwrap();

        create_token_account(
            &mut context,
            &fake_account,
            &fake_mint.pubkey(),
            &payer_pubkey,
        )
        .await
        .unwrap();

        mint_tokens(
            &mut context,
            &fake_mint.pubkey(),
            &fake_account.pubkey(),
            1,
            &payer_pubkey,
            None,
        )
        .await
        .unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[instruction::mint_new_edition_from_master_edition_via_token(
                id(),
                test_edition_marker.new_metadata_pubkey,
                test_edition_marker.new_edition_pubkey,
                test_edition_marker.master_edition_pubkey,
                fake_mint.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                context.payer.pubkey(),
                fake_account.pubkey(),
                context.payer.pubkey(),
                test_edition_marker.metadata_pubkey,
                test_edition_marker.metadata_mint_pubkey,
                test_edition_marker.edition,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &context.payer, &context.payer],
            context.last_blockhash,
        );

        let result = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        assert_custom_error!(result, MetadataError::TokenAccountMintMismatchV2);
    }

    #[tokio::test]
    async fn fail_edition_already_initialized() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 1);
        let test_edition_marker1 = EditionMarker::new(&test_metadata, &test_master_edition, 1);

        test_metadata
            .create(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                0,
            )
            .await
            .unwrap();

        test_master_edition
            .create(&mut context, Some(10))
            .await
            .unwrap();

        test_edition_marker.create(&mut context).await.unwrap();
        let result = test_edition_marker1.create(&mut context).await.unwrap_err();
        assert_custom_error!(result, MetadataError::AlreadyInitialized);
    }

    #[tokio::test]
    async fn fail_to_mint_edition_override_0() {
        let mut context = program_test().start_with_context().await;
        let test_metadata = Metadata::new();
        let test_master_edition = MasterEditionV2::new(&test_metadata);
        let test_edition_marker = EditionMarker::new(&test_metadata, &test_master_edition, 0);

        test_metadata
            .create(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                0,
            )
            .await
            .unwrap();

        test_master_edition
            .create(&mut context, Some(0))
            .await
            .unwrap();

        let result = test_edition_marker.create(&mut context).await.unwrap_err();
        assert_custom_error!(result, MetadataError::EditionOverrideCannotBeZero);
    }

    #[tokio::test]
    async fn increment_master_edition_supply() {
        // If the edition number being minted is less than the current supply, nothing should happen,
        // but if it's greater than the current supply, the supply amount should be increased by 1.
        let mut context = program_test().start_with_context().await;

        let original_nft = Metadata::new();
        original_nft
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let master_edition = MasterEditionV2::new(&original_nft);
        master_edition
            .create_v3(&mut context, Some(10))
            .await
            .unwrap();
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 1);
        print_edition.create(&mut context).await.unwrap();

        // Metadata, Print Edition and token account exist.
        assert!(print_edition.exists_on_chain(&mut context).await);

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 1);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 5 and supply should go up to 2.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 5);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 2);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 4 and supply should go up to 3.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 4);
        print_edition.create(&mut context).await.unwrap();

        let mut master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let mut master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 3);
        assert!(master_edition_struct.max_supply == Some(10));

        // Simulate a collection where there are are missing editions with numbers lower than the current
        // supply value and ensure they can still be minted.
        master_edition_struct.supply = 8;
        let mut data = master_edition_struct.try_to_vec().unwrap();
        let filler = vec![0u8; MAX_MASTER_EDITION_LEN - data.len()];
        data.extend_from_slice(&filler[..]);
        master_edition_account.data = data;

        let master_edition_shared_data: AccountSharedData = master_edition_account.into();
        context.set_account(&master_edition.pubkey, &master_edition_shared_data);

        assert!(master_edition_struct.supply == 8);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 2, this will succeed but supply will incremement.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 2);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 9);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint edition number 10 and supply should increase by 1 to 10.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 10);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 10);
        assert!(master_edition_struct.max_supply == Some(10));

        // Mint another edition and it should succeed, but supply should stay the same since it's already reached max supply.
        // This allows minting missing editions even when the supply has erraneously reached
        // the max supply.
        let print_edition = EditionMarker::new(&original_nft, &master_edition, 6);
        print_edition.create(&mut context).await.unwrap();

        let master_edition_account = context
            .banks_client
            .get_account(master_edition.pubkey)
            .await
            .unwrap()
            .unwrap();

        let master_edition_struct: ProgramMasterEdition =
            ProgramMasterEdition::safe_deserialize(&master_edition_account.data).unwrap();

        assert!(master_edition_struct.supply == 10);
        assert!(master_edition_struct.max_supply == Some(10));
    }
}
