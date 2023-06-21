// use database::Database;
// use gears::{
//     baseapp::{BankKeyProvider, MicroContext},
//     types::Context,
// };
// use proto_messages::cosmos::bank::v1beta1::MsgSend;
// use store::StoreKey;
// use strum::IntoEnumIterator;

// pub struct MsgTypeOne {}
// pub struct MsgTypeTwo {}

// pub fn process_message_type_one<T: Database>(
//     ctx: &mut Context<T>,
//     msg: MsgTypeOne,
// ) -> Result<(), String> {
//     return Ok(());
// }

// pub fn process_message_type_two<T: Database>(
//     ctx: &mut Context<T>,
//     msg: MsgTypeTwo,
// ) -> Result<(), String> {
//     return Ok(());
// }

// pub enum ModuleMsg {
//     TypeOne(MsgTypeOne),
//     TypeTwo(MsgTypeTwo),
// }

// impl ModuleMsg {
//     pub fn from_raw(url: &str) -> ModuleMsg {
//         match url {
//             "one" => ModuleMsg::TypeOne(MsgTypeOne {}),
//             _ => ModuleMsg::TypeTwo(MsgTypeTwo {}),
//         }
//     }
// }

// pub fn router<T: Database>(msg: ModuleMsg, ctx: &mut Context<T>) -> Result<(), String> {
//     match msg {
//         ModuleMsg::TypeOne(msg) => process_message_type_one(ctx, msg),
//         ModuleMsg::TypeTwo(msg) => process_message_type_two(ctx, msg),
//     }
// }

// //############################
// // Keeper

// use core::hash::Hash;

// fn send_coins<T: Database, S: Hash + Eq + IntoEnumIterator + BankKeyProvider + StoreKey>(
//     ctx: &mut MicroContext<T, S>,
//     msg: MsgSend,
// ) -> Result<(), String> {
//     let bank_store = ctx.get_mutable_kv_store(S::get_bank_key());
//     Ok(())

//     // let bank_store = ctx.get_mutable_kv_store(Store::Bank);
//     // let mut events = vec![];

//     // let from_address = msg.from_address;
//     // let to_address = msg.to_address;

//     // for send_coin in msg.amount {
//     //     let mut from_account_store = Bank::get_address_balances_store(bank_store, &from_address);
//     //     let from_balance = from_account_store
//     //         .get(send_coin.denom.to_string().as_bytes())
//     //         .ok_or(AppError::Send("Insufficient funds".into()))?;

//     //     let mut from_balance: Coin = Coin::decode::<Bytes>(from_balance.to_owned().into())
//     //         .expect("invalid data in database - possible database corruption");

//     //     if from_balance.amount < send_coin.amount {
//     //         return Err(AppError::Send("Insufficient funds".into()));
//     //     }

//     //     from_balance.amount = from_balance.amount - send_coin.amount;

//     //     from_account_store.set(
//     //         send_coin.denom.clone().to_string().into(),
//     //         from_balance.encode_vec(),
//     //     );

//     //     //TODO: if balance == 0 then denom should be removed from store

//     //     let mut to_account_store = Bank::get_address_balances_store(bank_store, &to_address);
//     //     let to_balance = to_account_store.get(send_coin.denom.to_string().as_bytes());

//     //     let mut to_balance: Coin = match to_balance {
//     //         Some(to_balance) => Coin::decode::<Bytes>(to_balance.to_owned().into())
//     //             .expect("invalid data in database - possible database corruption"),
//     //         None => Coin {
//     //             denom: send_coin.denom.clone(),
//     //             amount: Uint256::zero(),
//     //         },
//     //     };

//     //     to_balance.amount = to_balance.amount + send_coin.amount;

//     //     to_account_store.set(send_coin.denom.to_string().into(), to_balance.encode_vec());

//     //     events.push(Event::new(
//     //         "transfer",
//     //         vec![
//     //             ("recipient", String::from(to_address.clone())).index(),
//     //             ("sender", String::from(from_address.clone())).index(),
//     //             ("amount", send_coin.amount.into()).index(),
//     //         ],
//     //     ));
//     // }

//     // ctx.append_events(events);

//     // return Ok(());
// }
