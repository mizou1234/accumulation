use support::{decl_module, decl_storage, decl_event, StorageMap, dispatch::Result,ensure,StorageValue,traits::Currency};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};
use parity_codec::{Encode, Decode};
use rstd::vec::Vec;


//定义结构体,hash用来存放用户名称和地址BU
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Buyer<Hash, Balance> {
    id:Hash,
    price:Balance,
    amount: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Seller<Hash, Balance> {
    id:Hash,
    price:Balance,
    amount: u64,
}

pub trait Trait: system::Trait+balances::Trait{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// 	/// The overarching event type.
// 	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
// }

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as mymodulestorage {
		Buyers get(buyers): map T::AccountId =>Buyer<T::Hash, T::Balance>;
		Sellers get(sellers): map T::AccountId =>Seller<T::Hash, T::Balance>;
		BidLedger get(bidledger):Vec<Buyer<T::Hash, T::Balance>>;
		AskLedger get(askledger):Vec<Seller<T::Hash, T::Balance>>;
		BuyersArray get(buyerid): map Buyer<T::Hash, T::Balance> =>T::AccountId;
		SellersArray get(sellerid): map Seller<T::Hash, T::Balance> =>T::AccountId;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		//买家参与限价交易
		pub fn buy_limitOrder(origin,buyer_price:T::Balance,buyer_amount:u64) -> Result {
			let sender = ensure_signed(origin)?;
			let id = <T as system::Trait>::Hashing::hash_of(&sender);
			Self::submit_buyer(sender,id,buyer_price,buyer_amount);	
			Ok(())
		}
		//卖家参与限价交易
		pub fn sell_limitOrder(origin,seller_price:T::Balance,seller_amount:u64) -> Result {
			let sender = ensure_signed(origin)?;
			let id = <T as system::Trait>::Hashing::hash_of(&sender);
			Self::submit_seller(sender,id,seller_price,seller_amount);
			Ok(())
		}
		
		//买家撤单函数	    
	    pub fn buy_delete(origin) ->Result{
	    	let sender = ensure_signed(origin)?;
	    	Self::buy_deleteLimit(sender);
	    	Ok(())
	    }
	    //卖家撤单函数
	    pub fn sell_delete(origin) ->Result{
	    	let sender = ensure_signed(origin)?;
	    	Self::sell_deleteLimit(sender);
	    	Ok(())
	    }	    

        //买家修改限价订单
        pub fn buy_changeLimit(origin,buyer_id:T::Hash,buyer_price:T::Balance,buyer_amount:u64) -> Result{
        	let sender = ensure_signed(origin)?;
        	let new_change_buyer = Buyer{
				id: buyer_id,
	            price: buyer_price,
	            amount: buyer_amount,
			};
        	Self::buy_deleteLimit(sender.clone());
        	Self::submit_buyer(sender,buyer_id,buyer_price,buyer_amount);
        	Ok(())
        }
        //卖家修改限价订单
        pub fn sell_changeLimit(origin,seller_id:T::Hash,seller_price:T::Balance,seller_amount:u64) -> Result{
        	let sender = ensure_signed(origin)?;
         	let new_change_seller = Seller{
				id: seller_id,
	            price: seller_price,
	            amount: seller_amount,
			}; 
        	Self::sell_deleteLimit(sender.clone());
        	Self::submit_seller(sender,seller_id,seller_price,seller_amount);
        	Ok(())
        }
        
        //买家参与市价交易，并出清
        pub fn buy_marketOrder(origin,buyer_amount:u64) ->Result{
			let sender = ensure_signed(origin)?;
			let mut _amount = buyer_amount;
			let mut askledger=Self::askledger();
			let mut ask_index=askledger.iter().len()-1;
			while ask_index>=0 && _amount != 0 {
				let mut _sellerid=Self::sellerid(askledger[ask_index].clone());
				if _amount<askledger[ask_index].amount {
					let mut payamount1 = askledger[ask_index].price*<T::Balance as As<u64>>::sa(_amount);//payamount指交易额
					<balances::Module<T> as Currency<_>>::transfer(&sender, &mut _sellerid, payamount1)?;//use the transfer fn
					askledger[ask_index].amount -= _amount;
					Self::submit_seller(_sellerid,askledger[ask_index].id,askledger[ask_index].price,askledger[ask_index].amount);
					break;
				}
				if _amount>=askledger[ask_index].amount {
					let mut payamount2 = askledger[ask_index].price*<T::Balance as As<u64>>::sa(_amount);//payamount指交易额
					<balances::Module<T> as Currency<_>>::transfer(&sender, &mut _sellerid, payamount2)?;//use the transfer fn
					_amount-=askledger[ask_index].amount;
					askledger[ask_index].amount=0;
					ask_index-=1;
				}
			}
			for i in 0..askledger.iter().len(){
					if askledger[i].amount==0{
					askledger.remove(i);
				}	             	
			}
			<AskLedger<T>>::put(askledger);
			
        	Ok(())
        }
		//卖家参与市价交易，并出清
		pub fn sell_marketOrder(origin,seller_amount:u64) ->Result{
			let sender = ensure_signed(origin)?;
			let mut _amount = seller_amount;
			let mut bidledger=Self::bidledger();
			let mut bid_index=bidledger.iter().len()-1;
			while bid_index>=0 && _amount != 0 {
				let mut _buyerid=Self::buyerid(bidledger[bid_index].clone());
				if _amount<bidledger[bid_index].amount {
					let mut payamount1 = bidledger[bid_index].price*<T::Balance as As<u64>>::sa(_amount);//payamount指交易额
					<balances::Module<T> as Currency<_>>::transfer(&sender, &mut _buyerid, payamount1)?;//use the transfer fn
					bidledger[bid_index].amount -= _amount;
					Self::submit_buyer(_buyerid,bidledger[bid_index].id,bidledger[bid_index].price,bidledger[bid_index].amount);
					break;
				}
				if _amount>=bidledger[bid_index].amount {
					let mut payamount2 = bidledger[bid_index].price*<T::Balance as As<u64>>::sa(_amount);//payamount指交易额
					<balances::Module<T> as Currency<_>>::transfer(&sender, &mut _buyerid, payamount2)?;//use the transfer fn
					_amount-=bidledger[bid_index].amount;
					bidledger[bid_index].amount=0;
					bid_index-=1;
				}
			}
			for i in 0..bidledger.iter().len(){
					if bidledger[i].amount==0{
					bidledger.remove(i);
				}	             	
			}
			<BidLedger<T>>::put(bidledger);
			
			Ok(())
		}
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;
	}
}


impl<T: Trait> Module<T>{
   //获得bid队列最优报价 不知道impl里的函数用户可以调用吗
	pub fn bid_revealPrice() ->T::Balance{
		let bidledger=Self::bidledger();
		let length=bidledger.iter().len();
		bidledger[length].price
	}
	//获得ask队列最优报价
	pub fn ask_revealPrice() ->T::Balance{
		let askledger=Self::askledger();
		let length=askledger.iter().len();
		askledger[length].price
	}
	//买家撤单
    pub fn buy_deleteLimit(from:T::AccountId) ->Result{
    	let mut bidledger=Self::bidledger();
            for i in 0..bidledger.iter().len(){
            	    let mut sender=Self::buyerid(bidledger[i].clone());
	            	if sender==from{
                    bidledger.remove(i);
                   break;
             	}	             	
             }
        <BidLedger<T>>::put(bidledger);
		// <Buyers<T>>::pop(&from);
		// <BuyersArray<T>>::pop(&from);
		Ok(())
    }
    //卖家撤单函数
    pub fn sell_deleteLimit(from:T::AccountId) ->Result{
    	let mut askledger=Self::askledger();
            for i in 0..askledger.iter().len(){
            	    let mut _sellerid=Self::sellerid(askledger[i].clone());
	            	if _sellerid==from{
                    askledger.remove(i);
                    break;
             	}	             	
             }
        <AskLedger<T>>::put(askledger);
		Ok(())
    }

    //提交买家信息    
	pub fn submit_buyer(from:T::AccountId,buyer_id:T::Hash,buyer_price:T::Balance,buyer_amount:u64) -> Result {
		let new_buyer = Buyer{
			id: buyer_id,
            price: buyer_price,
            amount: buyer_amount,
		};
        
        let mut bidledger=Self::bidledger();
        if bidledger.iter().len()>0{

         	let mut flag=0;
            for i in 0..bidledger.iter().len(){
	            	if bidledger[i].price>buyer_price{
            		flag=1;
                    bidledger.insert(i,new_buyer.clone());
                   break;
             	}	             	
            }
	            if flag==0 {bidledger.push(new_buyer.clone());}
         }else{
         	    bidledger.push(new_buyer.clone());
         }
		<BidLedger<T>>::put(bidledger);
		<Buyers<T>>::insert(&from,new_buyer.clone());
		<BuyersArray<T>>::insert(new_buyer.clone(),&from);
	
		Ok(())
	}		
	//提交卖家信息
    pub fn submit_seller(from:T::AccountId,seller_id:T::Hash,seller_price:T::Balance,seller_amount:u64) -> Result {
		let new_seller = Seller{
			id: seller_id,
            price: seller_price,
            amount: seller_amount,
		};
        
        let mut askledger=Self::askledger();
        if askledger.iter().len()>0{

         	let mut flag=0;
            for i in 0..askledger.iter().len(){
	            	if askledger[i].price<seller_price{
            		flag=1;
                    askledger.insert(i,new_seller.clone());
                   break;
             	}	             	
            }
	            if flag==0 {askledger.push(new_seller.clone());}
         }else{
         	    askledger.push(new_seller.clone());
         }
		<AskLedger<T>>::put(askledger);
		<Sellers<T>>::insert(&from,new_seller.clone());
		<SellersArray<T>>::insert(new_seller.clone(),&from);
		Ok(())
	}
	
	
		
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type mymodule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(mymodule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(mymodule::something(), Some(42));
		});
	}
}
