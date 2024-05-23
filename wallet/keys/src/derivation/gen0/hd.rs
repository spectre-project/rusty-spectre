use crate::derivation::traits::*;
use crate::imports::*;
use hmac::Mac;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use spectre_addresses::{Address, Prefix as AddressPrefix, Version as AddressVersion};
use spectre_bip32::types::{ChainCode, HmacSha512, KeyFingerprint, PublicKeyBytes, KEY_SIZE};
use spectre_bip32::{
    AddressType, ChildNumber, DerivationPath, ExtendedKey, ExtendedKeyAttrs, ExtendedPrivateKey, ExtendedPublicKey, Prefix,
    PrivateKey, PublicKey, SecretKey, SecretKeyExt,
};
use std::fmt::Debug;

fn get_fingerprint<K>(private_key: &K) -> KeyFingerprint
where
    K: PrivateKey,
{
    let public_key_bytes = private_key.public_key().to_bytes();

    let digest = Ripemd160::digest(Sha256::digest(public_key_bytes));
    digest[..4].try_into().expect("digest truncated")
}

struct Inner {
    /// Derived public key
    public_key: secp256k1::PublicKey,
    /// Extended key attributes.
    attrs: ExtendedKeyAttrs,
    #[allow(dead_code)]
    fingerprint: KeyFingerprint,
    hmac: HmacSha512,
}

impl Inner {
    fn new(public_key: secp256k1::PublicKey, attrs: ExtendedKeyAttrs, hmac: HmacSha512) -> Self {
        Self { public_key, fingerprint: public_key.fingerprint(), hmac, attrs }
    }
}

#[derive(Clone)]
pub struct PubkeyDerivationManagerV0 {
    inner: Arc<Mutex<Option<Inner>>>,
    index: Arc<Mutex<u32>>,
    cache: Arc<Mutex<HashMap<u32, secp256k1::PublicKey>>>,
    use_cache: Arc<AtomicBool>,
}

impl PubkeyDerivationManagerV0 {
    pub fn new(
        public_key: secp256k1::PublicKey,
        attrs: ExtendedKeyAttrs,
        fingerprint: KeyFingerprint,
        hmac: HmacSha512,
        index: u32,
        use_cache: bool,
    ) -> Result<Self> {
        let wallet = Self {
            index: Arc::new(Mutex::new(index)),
            inner: Arc::new(Mutex::new(Some(Inner { public_key, attrs, fingerprint, hmac }))),
            cache: Arc::new(Mutex::new(HashMap::new())),
            use_cache: Arc::new(AtomicBool::new(use_cache)),
        };

        Ok(wallet)
    }

    fn set_key(&self, public_key: secp256k1::PublicKey, attrs: ExtendedKeyAttrs, hmac: HmacSha512, index: Option<u32>) {
        *self.cache.lock().unwrap() = HashMap::new();
        let new_inner = Inner::new(public_key, attrs, hmac);
        {
            *self.index.lock().unwrap() = index.unwrap_or(0);
        }
        let mut locked = self.opt_inner();
        if let Some(inner) = locked.as_mut() {
            inner.public_key = new_inner.public_key;
            inner.fingerprint = new_inner.fingerprint;
            inner.hmac = new_inner.hmac;
            inner.attrs = new_inner.attrs;
        } else {
            *locked = Some(new_inner)
        }
    }

    fn remove_key(&self) {
        *self.opt_inner() = None;
    }

    fn opt_inner(&self) -> MutexGuard<Option<Inner>> {
        self.inner.lock().unwrap()
    }

    fn public_key_(&self) -> Result<secp256k1::PublicKey> {
        let locked = self.opt_inner();
        let inner = locked
            .as_ref()
            .ok_or(crate::error::Error::Custom("PubkeyDerivationManagerV0 initialization is pending (Error: 101).".into()))?;
        Ok(inner.public_key)
    }
    fn index_(&self) -> Result<u32> {
        // let locked = self.opt_inner();
        // let inner =
        //     locked.as_ref().ok_or(crate::error::Error::Custom("PubkeyDerivationManagerV0 initialization is pending.".into()))?;
        // Ok(inner.index)
        Ok(*self.index.lock().unwrap())
    }

    fn use_cache(&self) -> bool {
        self.use_cache.load(Ordering::SeqCst)
    }

    pub fn cache(&self) -> Result<HashMap<u32, secp256k1::PublicKey>> {
        Ok(self.cache.lock()?.clone())
    }

    // pub fn derive_pubkey_range(&self, indexes: std::ops::Range<u32>) -> Result<Vec<secp256k1::PublicKey>> {
    //     let list = indexes.map(|index| self.derive_pubkey(index)).collect::<Vec<_>>();
    //     let keys = list.into_iter().collect::<Result<Vec<_>>>()?;
    //     // let keys = join_all(list).await.into_iter().collect::<Result<Vec<_>>>()?;
    //     Ok(keys)
    // }

    pub fn derive_pubkey_range(&self, indexes: std::ops::Range<u32>) -> Result<Vec<secp256k1::PublicKey>> {
        let use_cache = self.use_cache();
        let mut cache = self.cache.lock()?;
        let locked = self.opt_inner();
        let list: Vec<Result<secp256k1::PublicKey, crate::error::Error>> = if let Some(inner) = locked.as_ref() {
            indexes
                .map(|index| {
                    let (key, _chain_code) = WalletDerivationManagerV0::derive_public_key_child(
                        &inner.public_key,
                        ChildNumber::new(index, true)?,
                        inner.hmac.clone(),
                    )?;
                    //workflow_log::log_info!("use_cache: {use_cache}");
                    if use_cache {
                        //workflow_log::log_info!("cache insert: {:?}", key);
                        cache.insert(index, key);
                    }
                    Ok(key)
                })
                .collect::<Vec<_>>()
        } else {
            indexes
                .map(|index| {
                    if let Some(key) = cache.get(&index) {
                        Ok(*key)
                    } else {
                        Err(crate::error::Error::Custom("PubkeyDerivationManagerV0 initialization is pending  (Error: 102).".into()))
                    }
                })
                .collect::<Vec<_>>()
        };

        //let list = indexes.map(|index| self.derive_pubkey(index)).collect::<Vec<_>>();
        let keys = list.into_iter().collect::<Result<Vec<_>>>()?;
        // let keys = join_all(list).await.into_iter().collect::<Result<Vec<_>>>()?;
        Ok(keys)
    }

    pub fn derive_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        //let use_cache = self.use_cache();
        let locked = self.opt_inner();
        if let Some(inner) = locked.as_ref() {
            let (key, _chain_code) = WalletDerivationManagerV0::derive_public_key_child(
                &inner.public_key,
                ChildNumber::new(index, true)?,
                inner.hmac.clone(),
            )?;
            //workflow_log::log_info!("use_cache: {use_cache}");
            if self.use_cache() {
                //workflow_log::log_info!("cache insert: {:?}", key);
                self.cache.lock()?.insert(index, key);
            }
            return Ok(key);
        } else if let Some(key) = self.cache.lock()?.get(&index) {
            return Ok(*key);
        }

        Err(crate::error::Error::Custom("PubkeyDerivationManagerV0 initialization is pending  (Error: 102).".into()))
    }

    pub fn create_address(key: &secp256k1::PublicKey, prefix: AddressPrefix, _ecdsa: bool) -> Result<Address> {
        let payload = &key.to_bytes()[1..];
        let address = Address::new(prefix, AddressVersion::PubKey, payload);

        Ok(address)
    }

    pub fn public_key(&self) -> ExtendedPublicKey<secp256k1::PublicKey> {
        self.into()
    }

    pub fn attrs(&self) -> ExtendedKeyAttrs {
        let locked = self.opt_inner();
        let inner = locked.as_ref().expect("PubkeyDerivationManagerV0 initialization is pending (Error: 103).");
        inner.attrs.clone()
    }

    /// Serialize the raw public key as a byte array.
    pub fn to_bytes(&self) -> PublicKeyBytes {
        self.public_key().to_bytes()
    }

    /// Serialize this key as an [`ExtendedKey`].
    pub fn to_extended_key(&self, prefix: Prefix) -> ExtendedKey {
        let mut key_bytes = [0u8; KEY_SIZE + 1];
        key_bytes[..].copy_from_slice(&self.to_bytes());
        ExtendedKey { prefix, attrs: self.attrs().clone(), key_bytes }
    }

    pub fn to_string(&self) -> Zeroizing<String> {
        Zeroizing::new(self.to_extended_key(Prefix::XPUB).to_string())
    }
}

// #[wasm_bindgen]
impl PubkeyDerivationManagerV0 {
    // #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn get_public_key(&self) -> String {
        self.public_key().to_string(None)
    }
}

impl From<&PubkeyDerivationManagerV0> for ExtendedPublicKey<secp256k1::PublicKey> {
    fn from(inner: &PubkeyDerivationManagerV0) -> ExtendedPublicKey<secp256k1::PublicKey> {
        ExtendedPublicKey { public_key: inner.public_key_().unwrap(), attrs: inner.attrs().clone() }
    }
}

#[async_trait]
impl PubkeyDerivationManagerTrait for PubkeyDerivationManagerV0 {
    fn new_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.set_index(self.index()? + 1)?;
        self.current_pubkey()
    }

    fn index(&self) -> Result<u32> {
        self.index_()
    }

    fn set_index(&self, index: u32) -> Result<()> {
        *self.index.lock().unwrap() = index;
        Ok(())
    }

    fn current_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let index = self.index()?;
        //workflow_log::log_info!("current_pubkey");
        let key = self.derive_pubkey(index)?;

        Ok(key)
    }

    fn get_range(&self, range: std::ops::Range<u32>) -> Result<Vec<secp256k1::PublicKey>> {
        //workflow_log::log_info!("gen0: get_range {:?}", range);
        self.derive_pubkey_range(range)
    }

    fn get_cache(&self) -> Result<HashMap<u32, secp256k1::PublicKey>> {
        self.cache()
    }

    fn uninitialize(&self) -> Result<()> {
        self.remove_key();
        Ok(())
    }
}

#[derive(Clone)]
pub struct WalletDerivationManagerV0 {
    /// extended public key derived upto `m/<Purpose>'/972/<Account Index>'`
    extended_public_key: Option<ExtendedPublicKey<secp256k1::PublicKey>>,

    account_index: u64,
    /// receive address wallet
    receive_pubkey_manager: Arc<PubkeyDerivationManagerV0>,

    /// change address wallet
    change_pubkey_manager: Arc<PubkeyDerivationManagerV0>,
}

impl WalletDerivationManagerV0 {
    pub fn create_extended_key_from_xprv(xprv: &str, is_multisig: bool, account_index: u64) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let xprv_key = ExtendedPrivateKey::<SecretKey>::from_str(xprv)?;
        Self::derive_extended_key_from_master_key(xprv_key, is_multisig, account_index)
    }

    pub fn derive_extended_key_from_master_key(
        xprv_key: ExtendedPrivateKey<SecretKey>,
        _is_multisig: bool,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let attrs = xprv_key.attrs();

        let (extended_private_key, attrs) = Self::create_extended_key(*xprv_key.private_key(), attrs.clone(), account_index)?;

        Ok((extended_private_key, attrs))
    }

    fn create_extended_key(
        mut private_key: SecretKey,
        mut attrs: ExtendedKeyAttrs,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        // if is_multisig && cosigner_index.is_none() {
        //     return Err("cosigner_index is required for multisig path derivation".to_string().into());
        // }
        let purpose = 44; //if is_multisig { 45 } else { 44 };
        let path = format!("{purpose}'/972/{account_index}'");
        // if let Some(cosigner_index) = cosigner_index {
        //     path = format!("{path}/{}", cosigner_index)
        // }
        // if let Some(address_type) = address_type {
        //     path = format!("{path}/{}", address_type.index());
        // }
        //println!("path: {path}");
        let children = path.split('/');
        for child in children {
            (private_key, attrs) = Self::derive_private_key(&private_key, &attrs, child.parse::<ChildNumber>()?)?;
            //println!("ccc: {child}, public_key : {:?}, attrs: {:?}", private_key.get_public_key(), attrs);
        }

        Ok((private_key, attrs))
    }

    pub fn build_derivate_path(account_index: u64, address_type: Option<AddressType>) -> Result<DerivationPath> {
        let purpose = 44;
        let mut path = format!("m/{purpose}'/972/{account_index}'");
        if let Some(address_type) = address_type {
            path = format!("{path}/{}'", address_type.index());
        }
        let path = path.parse::<DerivationPath>()?;
        Ok(path)
    }

    pub fn receive_pubkey_manager(&self) -> &PubkeyDerivationManagerV0 {
        &self.receive_pubkey_manager
    }
    pub fn change_pubkey_manager(&self) -> &PubkeyDerivationManagerV0 {
        &self.change_pubkey_manager
    }

    pub fn create_pubkey_manager(
        private_key: &secp256k1::SecretKey,
        address_type: AddressType,
        attrs: &ExtendedKeyAttrs,
    ) -> Result<PubkeyDerivationManagerV0> {
        let (private_key, attrs, hmac) = Self::create_pubkey_manager_data(private_key, address_type, attrs)?;
        PubkeyDerivationManagerV0::new(
            private_key.get_public_key(),
            attrs.clone(),
            private_key.get_public_key().fingerprint(),
            hmac,
            0,
            true,
        )
    }

    pub fn create_pubkey_manager_data(
        private_key: &secp256k1::SecretKey,
        address_type: AddressType,
        attrs: &ExtendedKeyAttrs,
    ) -> Result<(secp256k1::SecretKey, ExtendedKeyAttrs, HmacSha512)> {
        let (private_key, attrs) = Self::derive_private_key(private_key, attrs, ChildNumber::new(address_type.index(), true)?)?;
        let hmac = Self::create_hmac(&private_key, &attrs, true)?;

        Ok((private_key, attrs, hmac))
    }

    pub fn derive_public_key(
        public_key: &secp256k1::PublicKey,
        attrs: &ExtendedKeyAttrs,
        child_number: ChildNumber,
    ) -> Result<(secp256k1::PublicKey, ExtendedKeyAttrs)> {
        //let fingerprint = public_key.fingerprint();
        let digest = Ripemd160::digest(Sha256::digest(&public_key.to_bytes()[1..]));
        let fingerprint = digest[..4].try_into().expect("digest truncated");

        let mut hmac = HmacSha512::new_from_slice(&attrs.chain_code).map_err(spectre_bip32::Error::Hmac)?;
        hmac.update(&public_key.to_bytes());

        let (key, chain_code) = Self::derive_public_key_child(public_key, child_number, hmac)?;

        let depth = attrs.depth.checked_add(1).ok_or(spectre_bip32::Error::Depth)?;

        let attrs = ExtendedKeyAttrs { parent_fingerprint: fingerprint, child_number, chain_code, depth };

        Ok((key, attrs))
    }

    fn derive_public_key_child(
        key: &secp256k1::PublicKey,
        child_number: ChildNumber,
        mut hmac: HmacSha512,
    ) -> Result<(secp256k1::PublicKey, ChainCode)> {
        hmac.update(&child_number.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (child_key, chain_code) = result.split_at(KEY_SIZE);

        // We should technically loop here if a `secret_key` is zero or overflows
        // the order of the underlying elliptic curve group, incrementing the
        // index, however per "Child key derivation (CKD) functions":
        // https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#child-key-derivation-ckd-functions
        //
        // > "Note: this has probability lower than 1 in 2^127."
        //
        // ...so instead, we simply return an error if this were ever to happen,
        // as the chances of it happening are vanishingly small.
        let key = key.derive_child(child_key.try_into()?)?;

        Ok((key, chain_code.try_into()?))
    }

    pub fn derive_key_by_path(
        xkey: &ExtendedPrivateKey<secp256k1::SecretKey>,
        path: DerivationPath,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let mut private_key = *xkey.private_key();
        let mut attrs = xkey.attrs().clone();
        for child in path {
            (private_key, attrs) = Self::derive_private_key(&private_key, &attrs, child)?;
        }

        Ok((private_key, attrs))
    }

    pub fn derive_private_key(
        private_key: &SecretKey,
        attrs: &ExtendedKeyAttrs,
        child_number: ChildNumber,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let fingerprint = get_fingerprint(private_key);

        let hmac = Self::create_hmac(private_key, attrs, child_number.is_hardened())?;

        let (private_key, chain_code) = Self::derive_key(private_key, child_number, hmac)?;

        let depth = attrs.depth.checked_add(1).ok_or(spectre_bip32::Error::Depth)?;

        let attrs = ExtendedKeyAttrs { parent_fingerprint: fingerprint, child_number, chain_code, depth };

        Ok((private_key, attrs))
    }

    fn derive_key(private_key: &SecretKey, child_number: ChildNumber, mut hmac: HmacSha512) -> Result<(SecretKey, ChainCode)> {
        hmac.update(&child_number.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (child_key, chain_code) = result.split_at(KEY_SIZE);

        // We should technically loop here if a `secret_key` is zero or overflows
        // the order of the underlying elliptic curve group, incrementing the
        // index, however per "Child key derivation (CKD) functions":
        // https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#child-key-derivation-ckd-functions
        //
        // > "Note: this has probability lower than 1 in 2^127."
        //
        // ...so instead, we simply return an error if this were ever to happen,
        // as the chances of it happening are vanishingly small.
        let private_key = private_key.derive_child(child_key.try_into()?)?;

        Ok((private_key, chain_code.try_into()?))
    }

    pub fn create_hmac<K>(private_key: &K, attrs: &ExtendedKeyAttrs, hardened: bool) -> Result<HmacSha512>
    where
        K: PrivateKey<PublicKey = secp256k1::PublicKey>,
    {
        let mut hmac = HmacSha512::new_from_slice(&attrs.chain_code).map_err(spectre_bip32::Error::Hmac)?;
        if hardened {
            hmac.update(&[0]);
            hmac.update(&private_key.to_bytes());
        } else {
            hmac.update(&private_key.public_key().to_bytes()[1..]);
        }

        Ok(hmac)
    }

    fn extended_public_key(&self) -> ExtendedPublicKey<secp256k1::PublicKey> {
        self.extended_public_key.clone().expect("WalletDerivationManagerV0 initialization is pending (Error: 104)")
    }

    /// Serialize the raw public key as a byte array.
    pub fn to_bytes(&self) -> PublicKeyBytes {
        self.extended_public_key().to_bytes()
    }

    pub fn attrs(&self) -> ExtendedKeyAttrs {
        self.extended_public_key().attrs().clone()
    }

    /// Serialize this key as a self-[`Zeroizing`] `String`.
    pub fn to_string(&self) -> Zeroizing<String> {
        let key = self.extended_public_key().to_string(Some(Prefix::KPUB));
        Zeroizing::new(key)
    }

    fn from_extended_private_key(private_key: secp256k1::SecretKey, account_index: u64, attrs: ExtendedKeyAttrs) -> Result<Self> {
        let receive_wallet = Self::create_pubkey_manager(&private_key, AddressType::Receive, &attrs)?;
        let change_wallet = Self::create_pubkey_manager(&private_key, AddressType::Change, &attrs)?;

        let extended_public_key = ExtendedPublicKey { public_key: private_key.get_public_key(), attrs };
        let wallet: WalletDerivationManagerV0 = Self {
            extended_public_key: Some(extended_public_key),
            account_index,
            receive_pubkey_manager: Arc::new(receive_wallet),
            change_pubkey_manager: Arc::new(change_wallet),
        };

        Ok(wallet)
    }

    pub fn create_uninitialized(
        account_index: u64,
        receive_keys: Option<HashMap<u32, secp256k1::PublicKey>>,
        change_keys: Option<HashMap<u32, secp256k1::PublicKey>>,
    ) -> Result<Self> {
        let receive_wallet = PubkeyDerivationManagerV0 {
            index: Arc::new(Mutex::new(0)),
            use_cache: Arc::new(AtomicBool::new(true)),
            cache: Arc::new(Mutex::new(receive_keys.unwrap_or_default())),
            inner: Arc::new(Mutex::new(None)),
        };
        let change_wallet = PubkeyDerivationManagerV0 {
            index: Arc::new(Mutex::new(0)),
            use_cache: Arc::new(AtomicBool::new(true)),
            cache: Arc::new(Mutex::new(change_keys.unwrap_or_default())),
            inner: Arc::new(Mutex::new(None)),
        };
        let wallet = Self {
            extended_public_key: None,
            account_index,
            receive_pubkey_manager: Arc::new(receive_wallet),
            change_pubkey_manager: Arc::new(change_wallet),
        };

        Ok(wallet)
    }

    // set master key "xprvxxxxxx"
    pub fn set_key(&self, key: String, index: Option<u32>) -> Result<()> {
        let (private_key, attrs) = Self::create_extended_key_from_xprv(&key, false, self.account_index)?;

        let (private_key_, attrs_, hmac_) = Self::create_pubkey_manager_data(&private_key, AddressType::Receive, &attrs)?;
        self.receive_pubkey_manager.set_key(private_key_.get_public_key(), attrs_, hmac_, index);

        let (private_key_, attrs_, hmac_) = Self::create_pubkey_manager_data(&private_key, AddressType::Change, &attrs)?;
        self.change_pubkey_manager.set_key(private_key_.get_public_key(), attrs_, hmac_, index);

        Ok(())
    }

    pub fn remove_key(&self) -> Result<()> {
        self.receive_pubkey_manager.remove_key();
        self.change_pubkey_manager.remove_key();
        Ok(())
    }
}

impl Debug for WalletDerivationManagerV0 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletAccount")
            .field("depth", &self.attrs().depth)
            .field("child_number", &self.attrs().child_number)
            .field("chain_code", &faster_hex::hex_string(&self.attrs().chain_code))
            .field("public_key", &faster_hex::hex_string(&self.to_bytes()))
            .field("parent_fingerprint", &self.attrs().parent_fingerprint)
            .finish()
    }
}

#[async_trait]
impl WalletDerivationManagerTrait for WalletDerivationManagerV0 {
    /// build wallet from root/master private key
    fn from_master_xprv(xprv: &str, _is_multisig: bool, account_index: u64, _cosigner_index: Option<u32>) -> Result<Self> {
        let xprv_key = ExtendedPrivateKey::<SecretKey>::from_str(xprv)?;
        let attrs = xprv_key.attrs();

        let (extended_private_key, attrs) = Self::create_extended_key(*xprv_key.private_key(), attrs.clone(), account_index)?;

        let wallet = Self::from_extended_private_key(extended_private_key, account_index, attrs)?;

        Ok(wallet)
    }

    fn from_extended_public_key_str(_xpub: &str, _cosigner_index: Option<u32>) -> Result<Self> {
        unreachable!();
    }

    fn from_extended_public_key(
        _extended_public_key: ExtendedPublicKey<secp256k1::PublicKey>,
        _cosigner_index: Option<u32>,
    ) -> Result<Self> {
        unreachable!();
    }

    fn receive_pubkey_manager(&self) -> Arc<dyn PubkeyDerivationManagerTrait> {
        self.receive_pubkey_manager.clone()
    }

    fn change_pubkey_manager(&self) -> Arc<dyn PubkeyDerivationManagerTrait> {
        self.change_pubkey_manager.clone()
    }

    #[inline(always)]
    fn new_receive_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.receive_pubkey_manager.new_pubkey()
    }

    #[inline(always)]
    fn new_change_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.change_pubkey_manager.new_pubkey()
    }

    #[inline(always)]
    fn receive_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.receive_pubkey_manager.current_pubkey()
    }

    #[inline(always)]
    fn change_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.change_pubkey_manager.current_pubkey()
    }

    #[inline(always)]
    fn derive_receive_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        self.receive_pubkey_manager.derive_pubkey(index)
    }

    #[inline(always)]
    fn derive_change_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        self.change_pubkey_manager.derive_pubkey(index)
    }

    fn initialize(&self, key: String, index: Option<u32>) -> Result<()> {
        self.set_key(key, index)?;
        Ok(())
    }
    fn uninitialize(&self) -> Result<()> {
        self.remove_key()?;
        Ok(())
    }
}

// #[cfg(test)]
// use super::hd_;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    //use super::hd_;
    use super::{PubkeyDerivationManagerV0, WalletDerivationManagerTrait, WalletDerivationManagerV0};
    use spectre_addresses::Prefix;

    fn gen0_receive_addresses() -> Vec<&'static str> {
        vec![
            "spectre:qqnklfz9safc78p30y5c9q6p2rvxhj35uhnh96uunklak0tjn2x5wz8n7akql",
            "spectre:qrd9efkvg3pg34sgp6ztwyv3r569qlc43wa5w8nfs302532dzj47k9fufq599",
            "spectre:qq9k5qju48zv4wuw6kjxdktyhm602enshpjzhp0lssdm73n7tl7l2latfpk9t",
            "spectre:qprpml6ytf4g85tgfhz63vks3hxq5mmc3ezxg5kc2aq3f7pmzedxxvgxt60ph",
            "spectre:qq7dzqep3elaf0hrqjg4t265px8k2eh2u4lmt78w4ph022gze2ahuvqt5fkwr",
            "spectre:qrx0uzsnagrzw259amacvae8lrlx2kl2h4dy8lg9p4dze2e5zkn0w3uwynnaf",
            "spectre:qr86w2yky258lrqxfc3w55hua6vsf6rshs3jq20ka00pvze34umek8pgeyq3x",
            "spectre:qq6gaad4ul2akwg3dz4jlqvmy3vjtkvdmfsfx6gxs76xafh2drwyvzcgggmnu",
            "spectre:qq9x43w57fg3l6jpyl9ytqf5k2czxqmtttecwfw6nu657hcsuf8zjle2rsr2v",
            "spectre:qr9pzwfce8va3c23m2lwc3up7xl2ngpqjwscs5wwu02nc0wlwgamj2wwk0rww",
            "spectre:qr3spcpku68mk9mjcq5qfk4at47aawxl2gz4kzndvu5jn4vzz79djlu2u20ap",
            "spectre:qp4v6d6lyn8k025fkal869sh6w7csw85gj930u9r5ml7anncqz6s72t6cr50h",
            "spectre:qzuas3nekcyl3uv6p8y5jrstchfweue0tpryttn6v0k4vc305rrej0pcs5j9y",
            "spectre:qpy00e8t4zd5ju8069zwsml2m7z3t607s87k0c66ud338ge682qwqfed608mx",
            "spectre:qrs04ra3yl33ejhx6dneqhm29ztdgmwrxw7ugatmecqqm9x5xvmrxrskuwh4f",
            "spectre:qq5qertse2y6p7vpjcef59ezuvhtdu028ucvvsn90htxvxycavregz6ftmhdt",
            "spectre:qrv30p7gatspj5x4u6drdux2ns5k08qxa3jmvh64ffxcqnxz925gsfludujpe",
            "spectre:qqfupvd2mm6rwswkxs0zp9lzttn690grhjx922wtpt7gfnsjdhk0zpzjnrxea",
            "spectre:qq2un0yhn4npc0rt2yjkp4aepz4j2rkryp59xlp6cvh0l5rqsndew4s3h28ee",
            "spectre:qzams4ymck03wfqj4xzvj39ufxl080h4jp32wa8hna2hua9kj6t6cfc082xyl",
        ]
    }

    fn gen0_change_addresses() -> Vec<&'static str> {
        vec![
            "spectre:qrp03wulr8z7cnr3lmwhpeuv5arthvnaydafgay8y3fg35fazclpcvhq5a8vf",
            "spectre:qpyum9jfp5ryf0wt9a36cpvp0tnj54kfnuqxjyad6eyn59qtg0cn6e062tz0z",
            "spectre:qp8p7vy9gtt6r5e77zaelgag68dvdf8kw4hts0mtmrcxm28sgjqdq6kc8pq6y",
            "spectre:qzsyzlp0xega2u82s5l235lschekxkpexju9jsrqscak2393wjkdc92cpqjh4",
            "spectre:qpxvpdfpr5jxlz3szrhdc8ggh33asyvg4w9lgvc207ju8zflmxsmg97z4aswy",
            "spectre:qz28qjteugexrat7c437hzv2wky5dwve862r2ahjuz8ry0m3jhd9zgllf26qm",
            "spectre:qz8cus3d2l4l4g3um93cy9nccmquvq62st2aan3xnet88cakhtljuq0kvyyuk",
            "spectre:qzczlu9crsn9f5n74sx3hnjv2aag83asrndc4crzg2eazngzlt0wqn63ud54q",
            "spectre:qqemqezzrgg99jp0tr8egwgnalqwma4z7jdnxjqqlyp6da0yktg5xn469cxqc",
            "spectre:qr0nfhyhqx6lt95lr0nf59lgskjqlsnq4tk4uwlxejxzj63f2g2acxttdw39j",
            "spectre:qqp0s3dacp46fvcaq5v2zl43smk2apzslawjqml6fhudfczp5d9n2h6rdgkpw",
            "spectre:qzac4rjzem4rvzr6kt2yjlq7whawzj9ra9calpw0euf507fdwuskqz0d90h24",
            "spectre:qrupjagxeqqzahlxtpraj5u4fd7x3p6l97npplge87pgeywkju47zkd7qqg2k",
            "spectre:qz208ms8heafvt90d28cpm3x7qvav87e3a2hgcz0e5t3d84xmlvcqknkj4c0k",
            "spectre:qq5357axc5ag8hzytf66p3fzw8d578h7xyfm4x4cpr3lp0wallglkv968tzjr",
            "spectre:qzsjhgefa98e4fsk58znu03mwzw7ymj7t4392l69kp0pgml2ymqm68xp2f3gh",
            "spectre:qplnwp0lxzwykmxrqphu62drmem2d09kfzplfek8z7cwt4s3vkkak6v9mc5ej",
            "spectre:qr4cm8smzgt8gzg33csv9mrsnvj9809ffun89cqsw65q3a37vmqx59afzm6et",
            "spectre:qpj0d7nznxp3nn2kyqsvm0ns38hzdk7dhj8g90cnrv9jda8xw5q2y3e5uree0",
            "spectre:qp4qt5cjrq73nuatnlwnk90lz5kqpd4mpqm53x7h3lpu74phz6zm57sn8ejp9",
        ]
    }

    #[tokio::test]
    async fn hd_wallet_gen0_set_key() {
        let master_xprv =
            "xprv9s21ZrQH143K3knsajkUfEx2ZVqX9iGm188iNqYL32yMVuMEFmNHudgmYmdU4NaNNKisDaGwV1kSGAagNyyGTTCpe1ysw6so31sx3PUCDCt";
        //println!("################################################################# 1111");
        let hd_wallet = WalletDerivationManagerV0::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();

        let hd_wallet_test = WalletDerivationManagerV0::create_uninitialized(0, None, None);
        assert!(hd_wallet_test.is_ok(), "Could not create empty wallet");
        let hd_wallet_test = hd_wallet_test.unwrap();

        let pubkey = hd_wallet_test.derive_receive_pubkey(0);
        assert!(pubkey.is_err(), "Should be error here");

        let res = hd_wallet_test.set_key(master_xprv.into(), None);
        assert!(res.is_ok(), "wallet_test.set_key() failed");

        for index in 0..20 {
            let pubkey = hd_wallet.derive_receive_pubkey(index).unwrap();
            let address1: String = PubkeyDerivationManagerV0::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();

            let pubkey = hd_wallet_test.derive_receive_pubkey(index).unwrap();
            let address2: String = PubkeyDerivationManagerV0::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();
            assert_eq!(address1, address2, "receive address at {index} failed");
        }

        let res = hd_wallet_test.remove_key();
        assert!(res.is_ok(), "wallet_test.remove_key() failed");

        let pubkey = hd_wallet_test.derive_receive_pubkey(0);
        assert!(pubkey.is_ok(), "Should be ok, as cache should return upto 0..20 keys");

        let pubkey = hd_wallet_test.derive_receive_pubkey(21);
        assert!(pubkey.is_err(), "Should be error here");
    }

    #[tokio::test]
    async fn hd_wallet_gen0() {
        let master_xprv =
            "xprv9s21ZrQH143K3knsajkUfEx2ZVqX9iGm188iNqYL32yMVuMEFmNHudgmYmdU4NaNNKisDaGwV1kSGAagNyyGTTCpe1ysw6so31sx3PUCDCt";
        //println!("################################################################# 1111");
        let hd_wallet = WalletDerivationManagerV0::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");

        //println!("################################################################# 2222");
        //let hd_wallet2 = hd_::WalletDerivationManagerV0::from_master_xprv(master_xprv, false, 0, None).await;
        //assert!(hd_wallet2.is_ok(), "Could not parse key1");

        let hd_wallet = hd_wallet.unwrap();
        //let hd_wallet2 = hd_wallet2.unwrap();

        let receive_addresses = gen0_receive_addresses();
        let change_addresses = gen0_change_addresses();

        //println!("$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$");
        //println!("hd_wallet1: {:?}", hd_wallet.receive_pubkey_manager().public_key());
        //println!("hd_wallet2: {:?}", hd_wallet2.receive_pubkey_manager.public_key());

        // let pubkey = hd_wallet2.derive_receive_pubkey(0).await.unwrap();
        // let address: String = hd_::PubkeyDerivationManagerV0::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();
        // assert_eq!(receive_addresses[0], address, "receive address at 0 failed $$$$ ");

        for index in 0..20 {
            let pubkey = hd_wallet.derive_receive_pubkey(index).unwrap();
            let address: String = PubkeyDerivationManagerV0::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();
            assert_eq!(receive_addresses[index as usize], address, "receive address at {index} failed");
            let pubkey = hd_wallet.derive_change_pubkey(index).unwrap();
            let address: String = PubkeyDerivationManagerV0::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();
            assert_eq!(change_addresses[index as usize], address, "change address at {index} failed");
        }
    }

    #[tokio::test]
    async fn generate_addresses_by_range() {
        let master_xprv =
            "xprv9s21ZrQH143K3knsajkUfEx2ZVqX9iGm188iNqYL32yMVuMEFmNHudgmYmdU4NaNNKisDaGwV1kSGAagNyyGTTCpe1ysw6so31sx3PUCDCt";

        let hd_wallet = WalletDerivationManagerV0::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();
        let pubkeys = hd_wallet.receive_pubkey_manager().derive_pubkey_range(0..20).unwrap();
        let addresses_receive = pubkeys
            .into_iter()
            .map(|k| PubkeyDerivationManagerV0::create_address(&k, Prefix::Mainnet, false).unwrap().to_string())
            .collect::<Vec<String>>();

        let pubkeys = hd_wallet.change_pubkey_manager().derive_pubkey_range(0..20).unwrap();
        let addresses_change = pubkeys
            .into_iter()
            .map(|k| PubkeyDerivationManagerV0::create_address(&k, Prefix::Mainnet, false).unwrap().to_string())
            .collect::<Vec<String>>();
        println!("receive addresses: {addresses_receive:#?}");
        println!("change addresses: {addresses_change:#?}");
        let receive_addresses = gen0_receive_addresses();
        let change_addresses = gen0_change_addresses();
        for index in 0..20 {
            assert_eq!(receive_addresses[index], addresses_receive[index], "receive address at {index} failed");
            assert_eq!(change_addresses[index], addresses_change[index], "change address at {index} failed");
        }
    }

    #[tokio::test]
    async fn generate_spectretest_addresses() {
        let receive_addresses = [
            "spectretest:qqz22l98sf8jun72rwh5rqe2tm8lhwtdxdmynrz4ypwak427qed5jcf5es549",
            "spectretest:qz880h6s4fwyumlslklt4jjwm7y5lcqyy8v5jc88gsncpuza0y76xcf5scved",
            "spectretest:qrxa994gjclvhnluxfet3056wwhrs02ptaj7gx04jlknjmlkmp9dxt5jnv4jn",
            "spectretest:qpqecy54rahaj4xadjm6my2a20fqmjysgrva3ya0nk2azhr90yrzyux9s6rxf",
            "spectretest:qzq3sc6jkr946fh3ycs0zg0vfz2jts54aa27rwy4ncqz9tm9ytnxsqv04x8aq",
            "spectretest:qq4vl7f82y2snr9warpy85f46sde0m0s8874p2rsq6p77fzccyflycn8jesu6",
            "spectretest:qq5kqzu76363zptuwt7kysqq9rmslcfypnyckqr4zjxfljx7p8mlw0gxlmqa4",
            "spectretest:qqad0qrj6y032jqxuygcyayvu2z8cza9hlvn8m89z3u6s6s8hg3dyen2yvj99",
            "spectretest:qpwkdpyf766ny56zuj47ax63l689wgg27rv90xr2pruk5px8sstcgphpud8hs",
            "spectretest:qpn0vug0j36xfdycq7nl6wczvqnhc22d6ayvhs646h76rv3pdpa87gd6gljmp",
            "spectretest:qz4c7eg9uernmsqmt429lvj5f85qsqzt6dgy8r53aefz39m77w2mglwvkfgr6",
            "spectretest:qqzfgqmmxrznec9hl35xwa8h6hs5mcr7lt7ep6j6373lxfq9jpj4630zfk9nc",
            "spectretest:qr9033gap4pscrhkwyp0cpmpy62a9pmcpqm2y4k29qqlktceulm7y3juh9rgw",
            "spectretest:qq3ktnql8uxwyj0kq6gq4vp8gm5ftnlvq0aphr55hl6u0u8dp49mqcq25wglv",
            "spectretest:qqrewmx4gpuekvk8grenkvj2hp7xt0c35rxgq383f6gy223c4ud5ssc7qqs76",
            "spectretest:qrhck7qaem2g2wtpqvjxtkpf87vd0ul8d8x70tu2zes3amcz70regnmsqya7w",
            "spectretest:qq4lnkxy9cdylkwnkhmz9z0cctfcqx8rzd4agdhzdvkmllrvc34nwtkxj4p4n",
            "spectretest:qzdt4wh0k63ndsv3m7t4n7flxu28qh3zdgh6ag684teervsfzzkcu44kslx6v",
            "spectretest:qqqng97tn6lfex3je7n0tr64e36zmzfyhpck2jeqts2ruatz3r5as28m7lkp9",
            "spectretest:qq2je8w0ltztef0ygljpcqx055kcxgxtsffwu7ujxzjfhk5p5rqlwzxzg4htv",
        ];

        let change_addresses = vec![
            "spectretest:qq3p8lvqyhzh37qgh2vf9u79l7h85pnmypg8z0tmp0tfl70zjm2cvwghaj35x",
            "spectretest:qpl00d5thmm3c5w3lj9cwx94dejjjx667rh3ey4sp0tkrmhsyd7rgp744qnez",
            "spectretest:qq407023vckl5u85u6w698fqu3ungs598z3xucc2mhr9vy0hug5vvzrmthj5e",
            "spectretest:qzl0qcvjfuwrrgzz83fuu272j7n9g03xfzp0g0f9jq5kll4rjfct549y807me",
            "spectretest:qp6l8n5meyut2yvpyw2dqrrcgc3t6jxflheh9j8s2f75quepdl4qvv2gcjnp5",
            "spectretest:qqw0uhr54kpyna0zrya6q7w2kya84ydgcvsdwaehayk8pn40d4y6sz776nmsw",
            "spectretest:qr5kjerrvnru7w49umrc0jtws6hpf7s22ur9nav0fsazs8kyy8ydwvuzzatfu",
            "spectretest:qqd8lyeya58hjym2xlw7th2wuenlptydmvzrzu53gxft0e2d844svr8r6l36p",
            "spectretest:qr0cs9lrdwjesuw5vf0x5rj78ecayphu60vt29smjerusqmec9w96e8mrnytk",
            "spectretest:qq089gr7p4rggwjqwh34mmdlsa357vprzl4q0dzn9c92egfs5aj5xucg6f5y3",
            "spectretest:qzs6m6nmkqczmxtjzptzzyl46nwwgq6hymk8jz3csg2h0lh0rpqjkwcgdnfzj",
            "spectretest:qr4k0fs6z47chukqv82walvyjmztd6czaqlk0kfdwr90rv3zwu5hjhq85kadq",
            "spectretest:qpgcua8savrpy7ggdxm0cq2uqgcd4a9skc39fld5avy3dvdcdsjssnwh436m2",
            "spectretest:qq2hllm2ff2rwgq3cyaczvusw5tr5ugfz2dtaedqxhuktz6sywveswt77gu58",
            "spectretest:qrr2a2lttpx8uaj0qtd80cl90h5qx7c9xgsdqzcfm2rntme9vuxpzk8tf44km",
            "spectretest:qqa8tjjr9ngudgh2gxyjevjazmgpx3v6zc3zn3aka38gm3erl6xx5p7ynqsc7",
            "spectretest:qqllkscqj7jd8tugj3rsl9r67evgandgnznekwl48cwp80jx6cut2awzus5a7",
            "spectretest:qq83n9wrk2ujn2hayyt74qfrctjp803csz5lsdzp0dslu7wue2ps5ftjxph4n",
            "spectretest:qz5qk6nvffsgdcujma3gq5rr2lr2q6yjw87n3w6asc0uj3rr8z8pk60qw35f0",
            "spectretest:qr55n5vkaq6lxcwzl6522nz86dj7ntl76nergy0u2j99v8w8lhyv6v0g5mmfm",
        ];

        let master_xprv =
            "xprv9s21ZrQH143K2rS8XAhiRk3NmkNRriFDrywGNQsWQqq8byBgBUt6A5uwTqYdZ3o5oDtKx7FuvNC1H1zPo7D5PXhszZTVgAvs79ehfTGESBh";

        let hd_wallet = WalletDerivationManagerV0::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();

        for index in 0..20 {
            let key = hd_wallet.derive_receive_pubkey(index).unwrap();
            //let address = Address::new(Prefix::Testnet, spectre_addresses::Version::PubKey, key.to_bytes());
            let address = PubkeyDerivationManagerV0::create_address(&key, Prefix::Testnet, false).unwrap();
            //receive_addresses.push(String::from(address));
            assert_eq!(receive_addresses[index as usize], address.to_string(), "receive address at {index} failed");
            let key = hd_wallet.derive_change_pubkey(index).unwrap();
            let address = PubkeyDerivationManagerV0::create_address(&key, Prefix::Testnet, false).unwrap();
            assert_eq!(change_addresses[index as usize], address.to_string(), "change address at {index} failed");
        }

        println!("receive_addresses: {receive_addresses:#?}");
    }
}
