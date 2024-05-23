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
// use wasm_bindgen::prelude::*;

fn get_fingerprint<K>(private_key: &K) -> KeyFingerprint
where
    K: PrivateKey,
{
    let public_key_bytes = private_key.public_key().to_bytes();

    let digest = Ripemd160::digest(Sha256::digest(public_key_bytes));
    digest[..4].try_into().expect("digest truncated")
}

#[derive(Clone)]
// #[wasm_bindgen(inspectable)]
pub struct PubkeyDerivationManager {
    /// Derived public key
    public_key: secp256k1::PublicKey,
    /// Extended key attributes.
    attrs: ExtendedKeyAttrs,
    #[allow(dead_code)]
    fingerprint: KeyFingerprint,
    hmac: HmacSha512,
    index: Arc<Mutex<u32>>,
}

impl PubkeyDerivationManager {
    pub fn new(
        public_key: secp256k1::PublicKey,
        attrs: ExtendedKeyAttrs,
        fingerprint: KeyFingerprint,
        hmac: HmacSha512,
        index: u32,
    ) -> Result<Self> {
        let wallet = Self { public_key, attrs, fingerprint, hmac, index: Arc::new(Mutex::new(index)) };

        Ok(wallet)
    }

    pub fn derive_pubkey_range(&self, indexes: std::ops::Range<u32>) -> Result<Vec<secp256k1::PublicKey>> {
        let list = indexes.map(|index| self.derive_pubkey(index)).collect::<Vec<_>>();
        let keys = list.into_iter().collect::<Result<Vec<_>>>()?;
        Ok(keys)
    }

    pub fn derive_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        let (key, _chain_code) = WalletDerivationManager::derive_public_key_child(&self.public_key, index, self.hmac.clone())?;
        Ok(key)
    }

    pub fn create_address(key: &secp256k1::PublicKey, prefix: AddressPrefix, ecdsa: bool) -> Result<Address> {
        let address = if ecdsa {
            let payload = &key.serialize();
            Address::new(prefix, AddressVersion::PubKeyECDSA, payload)
        } else {
            let payload = &key.x_only_public_key().0.serialize();
            Address::new(prefix, AddressVersion::PubKey, payload)
        };

        Ok(address)
    }

    pub fn public_key(&self) -> ExtendedPublicKey<secp256k1::PublicKey> {
        self.into()
    }

    pub fn attrs(&self) -> &ExtendedKeyAttrs {
        &self.attrs
    }

    /// Serialize the raw public key as a byte array.
    pub fn to_bytes(&self) -> PublicKeyBytes {
        self.public_key().to_bytes()
    }

    /// Serialize this key as an [`ExtendedKey`].
    pub fn to_extended_key(&self, prefix: Prefix) -> ExtendedKey {
        let mut key_bytes = [0u8; KEY_SIZE + 1];
        key_bytes[..].copy_from_slice(&self.to_bytes());
        ExtendedKey { prefix, attrs: self.attrs.clone(), key_bytes }
    }

    pub fn to_string(&self) -> Zeroizing<String> {
        Zeroizing::new(self.to_extended_key(Prefix::KPUB).to_string())
    }
}

// #[wasm_bindgen]
impl PubkeyDerivationManager {
    // #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn get_public_key(&self) -> String {
        self.public_key().to_string(None)
    }
}

impl From<&PubkeyDerivationManager> for ExtendedPublicKey<secp256k1::PublicKey> {
    fn from(inner: &PubkeyDerivationManager) -> ExtendedPublicKey<secp256k1::PublicKey> {
        ExtendedPublicKey { public_key: inner.public_key, attrs: inner.attrs().clone() }
    }
}

#[async_trait]
impl PubkeyDerivationManagerTrait for PubkeyDerivationManager {
    fn new_pubkey(&self) -> Result<secp256k1::PublicKey> {
        self.set_index(self.index()? + 1)?;
        self.current_pubkey()
    }

    fn index(&self) -> Result<u32> {
        Ok(*self.index.lock()?)
    }

    fn set_index(&self, index: u32) -> Result<()> {
        *self.index.lock()? = index;
        Ok(())
    }

    fn current_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let index = self.index()?;
        let key = self.derive_pubkey(index)?;

        Ok(key)
    }

    fn get_range(&self, range: std::ops::Range<u32>) -> Result<Vec<secp256k1::PublicKey>> {
        self.derive_pubkey_range(range)
    }
}

#[derive(Clone)]
pub struct WalletDerivationManager {
    /// extended public key derived upto `m/<Purpose>'/123456'/<Account Index>'`
    extended_public_key: ExtendedPublicKey<secp256k1::PublicKey>,

    /// receive address wallet
    receive_pubkey_manager: Arc<PubkeyDerivationManager>,

    /// change address wallet
    change_pubkey_manager: Arc<PubkeyDerivationManager>,
}

impl WalletDerivationManager {
    pub fn create_extended_key_from_xprv(xprv: &str, is_multisig: bool, account_index: u64) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let xprv_key = ExtendedPrivateKey::<SecretKey>::from_str(xprv)?;
        Self::derive_extended_key_from_master_key(xprv_key, is_multisig, account_index)
    }

    pub fn derive_extended_key_from_master_key(
        xprv_key: ExtendedPrivateKey<SecretKey>,
        is_multisig: bool,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let attrs = xprv_key.attrs();

        let (extended_private_key, attrs) =
            Self::create_extended_key(*xprv_key.private_key(), attrs.clone(), is_multisig, account_index)?;

        Ok((extended_private_key, attrs))
    }

    fn create_extended_key(
        mut private_key: SecretKey,
        mut attrs: ExtendedKeyAttrs,
        is_multisig: bool,
        account_index: u64,
    ) -> Result<(SecretKey, ExtendedKeyAttrs)> {
        let purpose = if is_multisig { 45 } else { 44 };
        let address_path = format!("{purpose}'/123456'/{account_index}'");
        let children = address_path.split('/');
        for child in children {
            (private_key, attrs) = Self::derive_private_key(&private_key, &attrs, child.parse::<ChildNumber>()?)?;
        }

        Ok((private_key, attrs))
    }

    pub fn build_derivate_path(
        is_multisig: bool,
        account_index: u64,
        cosigner_index: Option<u32>,
        address_type: Option<AddressType>,
    ) -> Result<DerivationPath> {
        if is_multisig && cosigner_index.is_none() {
            return Err("cosigner_index is required for multisig path derivation".to_string().into());
        }
        let purpose = if is_multisig { 45 } else { 44 };
        let mut path = format!("m/{purpose}'/123456'/{account_index}'");
        if let Some(cosigner_index) = cosigner_index {
            path = format!("{path}/{}", cosigner_index)
        }
        if let Some(address_type) = address_type {
            path = format!("{path}/{}", address_type.index());
        }
        let path = path.parse::<DerivationPath>()?;
        Ok(path)
    }

    pub fn receive_pubkey_manager(&self) -> &PubkeyDerivationManager {
        &self.receive_pubkey_manager
    }
    pub fn change_pubkey_manager(&self) -> &PubkeyDerivationManager {
        &self.change_pubkey_manager
    }

    pub fn derive_child_pubkey_manager(
        mut public_key: ExtendedPublicKey<secp256k1::PublicKey>,
        address_type: AddressType,
        cosigner_index: Option<u32>,
    ) -> Result<PubkeyDerivationManager> {
        if let Some(cosigner_index) = cosigner_index {
            public_key = public_key.derive_child(ChildNumber::new(cosigner_index, false)?)?;
        }

        public_key = public_key.derive_child(ChildNumber::new(address_type.index(), false)?)?;

        let mut hmac = HmacSha512::new_from_slice(&public_key.attrs().chain_code).map_err(spectre_bip32::Error::Hmac)?;
        hmac.update(&public_key.to_bytes());

        PubkeyDerivationManager::new(*public_key.public_key(), public_key.attrs().clone(), public_key.fingerprint(), hmac, 0)
    }

    pub fn derive_public_key(
        public_key: &secp256k1::PublicKey,
        attrs: &ExtendedKeyAttrs,
        index: u32,
    ) -> Result<(secp256k1::PublicKey, ExtendedKeyAttrs)> {
        let fingerprint = public_key.fingerprint();

        let mut hmac = HmacSha512::new_from_slice(&attrs.chain_code).map_err(spectre_bip32::Error::Hmac)?;
        hmac.update(&public_key.to_bytes());

        let (key, chain_code) = Self::derive_public_key_child(public_key, index, hmac)?;

        let depth = attrs.depth.checked_add(1).ok_or(spectre_bip32::Error::Depth)?;

        let attrs =
            ExtendedKeyAttrs { parent_fingerprint: fingerprint, child_number: ChildNumber::new(index, false)?, chain_code, depth };

        Ok((key, attrs))
    }

    fn derive_public_key_child(
        key: &secp256k1::PublicKey,
        index: u32,
        mut hmac: HmacSha512,
    ) -> Result<(secp256k1::PublicKey, ChainCode)> {
        let child_number = ChildNumber::new(index, false)?;
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
            hmac.update(&private_key.public_key().to_bytes());
        }

        Ok(hmac)
    }

    /// Serialize the raw public key as a byte array.
    pub fn to_bytes(&self) -> PublicKeyBytes {
        self.extended_public_key.to_bytes()
    }

    pub fn attrs(&self) -> &ExtendedKeyAttrs {
        self.extended_public_key.attrs()
    }

    /// Serialize this key as a self-[`Zeroizing`] `String`.
    pub fn to_string(&self, prefix: Option<Prefix>) -> Zeroizing<String> {
        let key = self.extended_public_key.to_string(Some(prefix.unwrap_or(Prefix::KPUB)));
        Zeroizing::new(key)
    }
}

impl Debug for WalletDerivationManager {
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
impl WalletDerivationManagerTrait for WalletDerivationManager {
    /// build wallet from root/master private key
    fn from_master_xprv(xprv: &str, is_multisig: bool, account_index: u64, cosigner_index: Option<u32>) -> Result<Self> {
        let xprv_key = ExtendedPrivateKey::<SecretKey>::from_str(xprv)?;
        let attrs = xprv_key.attrs();

        let (extended_private_key, attrs) =
            Self::create_extended_key(*xprv_key.private_key(), attrs.clone(), is_multisig, account_index)?;

        let extended_public_key = ExtendedPublicKey { public_key: extended_private_key.get_public_key(), attrs };

        let wallet = Self::from_extended_public_key(extended_public_key, cosigner_index)?;

        Ok(wallet)
    }

    fn from_extended_public_key_str(xpub: &str, cosigner_index: Option<u32>) -> Result<Self> {
        let extended_public_key = ExtendedPublicKey::<secp256k1::PublicKey>::from_str(xpub)?;
        let wallet = Self::from_extended_public_key(extended_public_key, cosigner_index)?;
        Ok(wallet)
    }

    fn from_extended_public_key(
        extended_public_key: ExtendedPublicKey<secp256k1::PublicKey>,
        cosigner_index: Option<u32>,
    ) -> Result<Self> {
        let receive_wallet = Self::derive_child_pubkey_manager(extended_public_key.clone(), AddressType::Receive, cosigner_index)?;

        let change_wallet = Self::derive_child_pubkey_manager(extended_public_key.clone(), AddressType::Change, cosigner_index)?;

        let wallet = Self {
            extended_public_key,
            receive_pubkey_manager: Arc::new(receive_wallet),
            change_pubkey_manager: Arc::new(change_wallet),
        };

        Ok(wallet)
    }

    fn receive_pubkey_manager(&self) -> Arc<dyn PubkeyDerivationManagerTrait> {
        self.receive_pubkey_manager.clone()
    }

    fn change_pubkey_manager(&self) -> Arc<dyn PubkeyDerivationManagerTrait> {
        self.change_pubkey_manager.clone()
    }

    #[inline(always)]
    fn new_receive_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.receive_pubkey_manager.new_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn new_change_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.change_pubkey_manager.new_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn receive_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.receive_pubkey_manager.current_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn change_pubkey(&self) -> Result<secp256k1::PublicKey> {
        let key = self.change_pubkey_manager.current_pubkey()?;
        Ok(key)
    }

    #[inline(always)]
    fn derive_receive_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        let key = self.receive_pubkey_manager.derive_pubkey(index)?;
        Ok(key)
    }

    #[inline(always)]
    fn derive_change_pubkey(&self, index: u32) -> Result<secp256k1::PublicKey> {
        let key = self.change_pubkey_manager.derive_pubkey(index)?;
        Ok(key)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::{PubkeyDerivationManager, WalletDerivationManager, WalletDerivationManagerTrait};
    use spectre_addresses::Prefix;

    fn gen1_receive_addresses() -> Vec<&'static str> {
        vec![
            "spectre:qp7mxlmfrf9xyfnpzvae7klfqpkx54nw9fxxmk6d9nfr0e9v6l2exgygpqnup",
            "spectre:qruqt5rjl6n2m28wy9p9tq0zm0rgvfgh4uw3jvgtu0s0azdv7zqdcthlq0llg",
            "spectre:qq4c84tkktggscj2lnrhep5yyrkcjlv54an496r8k9rkau9nfvhvw5qnu8knm",
            "spectre:qq9c5n0r6fuyje0up7nesqk772t0gxlw8rmvew6ka4qc64jf3vy9vvvz4053w",
            "spectre:qq07jg2h4jt6kgnyyvu38fuz6ug9xj3cuwhw7j3t9va4exfpzwx72fdavmydc",
            "spectre:qzalaygx32nlw6656fnw7u5dhl4kerln9e6snxrn2tuvs4vrd68jc55zad025",
            "spectre:qr9a7zgus5l05nqzhfamrcaewqd6qljtle4c20nj4vt7m96faekt5aynk0k68",
            "spectre:qp4x7j6da4vlf5zh3r4e6uszqpaaqt7q5gg34m0hga05jh33qcl26y6dwmrkv",
            "spectre:qpnduhadnf3a67axnve7e2d0yj535x5xagv3n2n6nx68tsedvqt6smn02ctrt",
            "spectre:qqzd3c8g8mdh9dspgngzxyejnpncrn62pglft27l02u6gus7cfnx6asr9hkjy",
            "spectre:qqmrjvvjfkhdlvfa5aw2zmzy8w00s5ky3dg9s85kg68e00h2rx9akn2t9ugdd",
            "spectre:qpjp9s3m7hjprl4zm8wjmcp5a98344f9aw4t0e2fzth34t9mxada5zknaju3h",
            "spectre:qrv754nq30t4duc6g538uajrvyxq2jcmn9umunhwj98es0kh5pzd6gcsq86z0",
            "spectre:qr2x4ssv4wup8azfafshh6hutrrk9crdem70qsdgxp5qdxwasavkyn79qupds",
            "spectre:qqma40ry0wac67zn5qrpvzuq8jvqlnh2q8cg6kdslykhvnu2yd6mgm05eqc5g",
            "spectre:qr8fnzec5wgx27zwv7krht0jq34vc3vgp0xw90580vw0gw7vnzs8zwys2fnkd",
            "spectre:qpstr0vsu8lqsht2c3v9pjnvurex6a3uz9rmr5wn3tkjx5d4u0ttjvmhjh0st",
            "spectre:qqmzy0l7gjh3af2mqkd76s0h87uxpfxm5we8cnglndrlvyg0447hjegsrxcp8",
            "spectre:qr6j5j22gr027e9x8w9a22rzsfqmysk8gc4ezrmvlj3afpsx2w7gu6c8capls",
            "spectre:qqkfsh33n2h75p9flcz3f7fmk9x5dpuj358l9pjf5a3cucmvyxzkkrrgyk5mq",
        ]
    }

    fn gen1_change_addresses() -> Vec<&'static str> {
        vec![
            "spectre:qp9hr5py5ayptv4r7e8tdtju7ffgk477p9y9zj5zx307npqgwjz3zpzm64my9",
            "spectre:qp3lk8cyheywqrjkfag8w6wpc2asw4aa7sjswjj9lp78yj6pwcjz5yvcsd7mf",
            "spectre:qptqpkt4rmp33vf9p6v0ty0pm5lmdc7h5dy4terq006mrd49vyjgxqmhvx8ur",
            "spectre:qqaf2v0gwx405tdy8gvqhgfrsunkm39vw7dz65a55vh20sh8lv5y7qh7c3uwy",
            "spectre:qzuu4jvzjfy59s2x26tv9r2chm79va946jkw7ck8yj5989ujymflgs32d4wc9",
            "spectre:qqdme3hy68vsv60zh4ul0kvecaazg0kr5je6vzkytxntcq2avmg97flm05m9g",
            "spectre:qrd03w0mn30d07h4pz5ayftl9y9hmqjcedf0446ftlzy7v7860d2ujgqr604q",
            "spectre:qp3wkulfgcgdtkr725dwc7qxflya7lp7cr7m8ykkxhgq7ghwgg3xv6dxgk0e0",
            "spectre:qrymwedga0j8lj6x5705deem3ya403lz0k35mt2mhf8mfzc7d48d7ru78fg69",
            "spectre:qqzt087fcs3vg3a2q3ydrm0q67sf05qnu8m8j8e9nkrvzrg558q25jgecetgw",
            "spectre:qpp8awyludwfm5arkzkcqjaeqth2zcj33z67eswh2e3egyq7hzm6wfr43vnr2",
            "spectre:qzwel0krm20mtq6dmq8uecushj50ydecqqnftsg3xrwe9ggsk5hzyjt7w29ca",
            "spectre:qpfy6kusnsjuu9gwp8u9h8a94al0rvuntcjvwh33dafh92qm246fkhhcslw98",
            "spectre:qrzdhd7shp2vvafv47w5ug9385jmeg0r7zfmmjqyn0zxy38huuqzqa475m2k7",
            "spectre:qq25ry8w59ypmlr0mws0tw0n4sqkeen0skzwvjeynqweud3fen60yruz9swz5",
            "spectre:qzmgwqh3a805ate4pv6fhv5e444vxdvn6u8nqv5zdrg3jg2dv0j5c67wk90df",
            "spectre:qru807246ca56zfanu773crwcvy5u0y293afdpup4p7pc96wx6p7696cynj9d",
            "spectre:qp5pa60pyfktyrx5nrkevshu94dtk0xk7scvxz3qyr0pkxpm630hjqxczha3q",
            "spectre:qq3djvcum0m3588m8z5y3q2rz677gf73f8m7eu2c5ktwk4e27enscx4ydcv8n",
            "spectre:qzvp6cpjv67s3urqwvudrk7gzcq9fu48yy42dtg0mqkcysqxky9cqxwnyfngx",
        ]
    }

    #[tokio::test]
    async fn hd_wallet_gen1() {
        let master_xprv =
            "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ";

        let hd_wallet = WalletDerivationManager::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();

        let receive_addresses = gen1_receive_addresses();
        let change_addresses = gen1_change_addresses();

        for index in 0..20 {
            let pubkey = hd_wallet.derive_receive_pubkey(index).unwrap();
            let address: String = PubkeyDerivationManager::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();
            assert_eq!(receive_addresses[index as usize], address, "receive address at {index} failed");
            let pubkey = hd_wallet.derive_change_pubkey(index).unwrap();
            let address: String = PubkeyDerivationManager::create_address(&pubkey, Prefix::Mainnet, false).unwrap().into();
            assert_eq!(change_addresses[index as usize], address, "change address at {index} failed");
        }
    }

    #[tokio::test]
    async fn wallet_from_mnemonic() {
        let mnemonic = "fringe ceiling crater inject pilot travel gas nurse bulb bullet horn segment snack harbor dice laugh vital cigar push couple plastic into slender worry";
        let mnemonic = spectre_bip32::Mnemonic::new(mnemonic, spectre_bip32::Language::English).unwrap();
        let xprv = spectre_bip32::ExtendedPrivateKey::<spectre_bip32::SecretKey>::new(mnemonic.to_seed("")).unwrap();
        let xprv_str = xprv.to_string(spectre_bip32::Prefix::KPRV).to_string();
        assert_eq!(
            xprv_str,
            "kprv5y2qurMHCsXYrpeDB395BY2DPKYHUGaCMpFAYRi1cmhwin1bWRyUXVbtTyy54FCGxPnnEvbK9WaiaQgkGS9ngGxmHy1bubZYY6MTokeYP2Q",
            "xprv not matched"
        );

        let wallet = WalletDerivationManager::from_master_xprv(&xprv_str, false, 0, None).unwrap();
        let xpub_str = wallet.to_string(Some(spectre_bip32::Prefix::KPUB)).to_string();
        assert_eq!(
            xpub_str,
            "kpub2KNbSheSoQpESaU4sYgt2mwVvhDHaJYMYQuzAx5pB27MSu4AFspv135ET8G4HfU1RoUsREa3H5Zk8Ew3JNdsmpZ5kb5ja38cRWWyDPYXbTd",
            "drived kpub not matched"
        );

        println!("Extended kpub: {}\n", xpub_str);
    }

    #[tokio::test]
    async fn address_test_by_ktrv() {
        let mnemonic = "hunt bitter praise lift buyer topic crane leopard uniform network inquiry over grain pass match crush marine strike doll relax fortune trumpet sunny silk";
        let mnemonic = spectre_bip32::Mnemonic::new(mnemonic, spectre_bip32::Language::English).unwrap();
        let xprv = spectre_bip32::ExtendedPrivateKey::<spectre_bip32::SecretKey>::new(mnemonic.to_seed("")).unwrap();
        let ktrv_str = xprv.to_string(spectre_bip32::Prefix::KTRV).to_string();
        assert_eq!(
            ktrv_str,
            "ktrv5himbbCxArFU2CHiEQyVHP1ABS1tA1SY88CwePzGeM8gHfWmkNBXehhKsESH7UwcxpjpDdMNbwtBfyPoZ7W59kYfVnUXKRgv8UguDns2FQb",
            "master ktrv not matched"
        );

        let wallet = WalletDerivationManager::from_master_xprv(&ktrv_str, false, 0, None).unwrap();
        let ktub_str = wallet.to_string(Some(spectre_bip32::Prefix::KTUB)).to_string();
        assert_eq!(
            ktub_str,
            "ktub23WYLhw7duhjZpKLgbsYpmsdUrghwtcaqSn84pDuH1WiiWkTQo8MU67JiY6MySf9KXJrEiVvFtEamSTykSCtDKDBbXToGJbMKx4pCXWuk8Z",
            "drived ktub not matched"
        );

        let key = wallet.derive_receive_pubkey(1).unwrap();
        let address = PubkeyDerivationManager::create_address(&key, Prefix::Testnet, false).unwrap().to_string();
        assert_eq!(address, "spectretest:qpkecaw7qs96xpe9xksten0m3wdef7e6w40t30x4ts49r5ujlm4wg3xyydr8v")
    }

    #[tokio::test]
    async fn generate_addresses_by_range() {
        let master_xprv =
            "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ";

        let hd_wallet = WalletDerivationManager::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();
        let pubkeys = hd_wallet.receive_pubkey_manager().derive_pubkey_range(0..20).unwrap();
        let addresses_receive = pubkeys
            .into_iter()
            .map(|k| PubkeyDerivationManager::create_address(&k, Prefix::Mainnet, false).unwrap().to_string())
            .collect::<Vec<String>>();

        let pubkeys = hd_wallet.change_pubkey_manager().derive_pubkey_range(0..20).unwrap();
        let addresses_change = pubkeys
            .into_iter()
            .map(|k| PubkeyDerivationManager::create_address(&k, Prefix::Mainnet, false).unwrap().to_string())
            .collect::<Vec<String>>();
        println!("receive addresses: {addresses_receive:#?}");
        println!("change addresses: {addresses_change:#?}");
        let receive_addresses = gen1_receive_addresses();
        let change_addresses = gen1_change_addresses();
        for index in 0..20 {
            assert_eq!(receive_addresses[index], addresses_receive[index], "receive address at {index} failed");
            assert_eq!(change_addresses[index], addresses_change[index], "change address at {index} failed");
        }
    }

    #[tokio::test]
    async fn generate_spectretest_addresses() {
        let receive_addresses = [
            "spectretest:qp7mxlmfrf9xyfnpzvae7klfqpkx54nw9fxxmk6d9nfr0e9v6l2exmgzdf6yz",
            "spectretest:qruqt5rjl6n2m28wy9p9tq0zm0rgvfgh4uw3jvgtu0s0azdv7zqdccm4vxk8t",
            "spectretest:qq4c84tkktggscj2lnrhep5yyrkcjlv54an496r8k9rkau9nfvhvw8veswltc",
            "spectretest:qq9c5n0r6fuyje0up7nesqk772t0gxlw8rmvew6ka4qc64jf3vy9vlqgexafd",
            "spectretest:qq07jg2h4jt6kgnyyvu38fuz6ug9xj3cuwhw7j3t9va4exfpzwx726phqjd4m",
            "spectretest:qzalaygx32nlw6656fnw7u5dhl4kerln9e6snxrn2tuvs4vrd68jc8cg3yxjh",
            "spectretest:qr9a7zgus5l05nqzhfamrcaewqd6qljtle4c20nj4vt7m96faekt5wge6xlzy",
            "spectretest:qp4x7j6da4vlf5zh3r4e6uszqpaaqt7q5gg34m0hga05jh33qcl26hk8zj2w0",
            "spectretest:qpnduhadnf3a67axnve7e2d0yj535x5xagv3n2n6nx68tsedvqt6sgl9x3zmg",
            "spectretest:qqzd3c8g8mdh9dspgngzxyejnpncrn62pglft27l02u6gus7cfnx6wuff7l28",
            "spectretest:qqmrjvvjfkhdlvfa5aw2zmzy8w00s5ky3dg9s85kg68e00h2rx9akqxpf4p4w",
            "spectretest:qpjp9s3m7hjprl4zm8wjmcp5a98344f9aw4t0e2fzth34t9mxada536e3m4f5",
            "spectretest:qrv754nq30t4duc6g538uajrvyxq2jcmn9umunhwj98es0kh5pzd6m56vwn6v",
            "spectretest:qr2x4ssv4wup8azfafshh6hutrrk9crdem70qsdgxp5qdxwasavkyqj0v4g4n",
            "spectretest:qqma40ry0wac67zn5qrpvzuq8jvqlnh2q8cg6kdslykhvnu2yd6mggr74f3vt",
            "spectretest:qr8fnzec5wgx27zwv7krht0jq34vc3vgp0xw90580vw0gw7vnzs8zag6xq6ww",
            "spectretest:qpstr0vsu8lqsht2c3v9pjnvurex6a3uz9rmr5wn3tkjx5d4u0ttjlha77xgg",
            "spectretest:qqmzy0l7gjh3af2mqkd76s0h87uxpfxm5we8cnglndrlvyg0447hj2y6003ey",
            "spectretest:qr6j5j22gr027e9x8w9a22rzsfqmysk8gc4ezrmvlj3afpsx2w7guf5d55g8n",
            "spectretest:qqkfsh33n2h75p9flcz3f7fmk9x5dpuj358l9pjf5a3cucmvyxzkks0zglarr",
        ];

        let master_xprv =
            "kprv5y2qurMHCsXYrNfU3GCihuwG3vMqFji7PZXajMEqyBkNh9UZUJgoHYBLTKu1eM4MvUtomcXPQ3Sw9HZ5ebbM4byoUciHo1zrPJBQfqpLorQ";

        let hd_wallet = WalletDerivationManager::from_master_xprv(master_xprv, false, 0, None);
        assert!(hd_wallet.is_ok(), "Could not parse key");
        let hd_wallet = hd_wallet.unwrap();

        //let mut receive_addresses = vec![]; //gen1_receive_addresses();
        //let change_addresses = gen1_change_addresses();

        for index in 0..20 {
            let key = hd_wallet.derive_receive_pubkey(index).unwrap();
            //let address = Address::new(Prefix::Testnet, spectre_addresses::Version::PubKey, key.to_bytes());
            let address = PubkeyDerivationManager::create_address(&key, Prefix::Testnet, false).unwrap();
            //receive_addresses.push(String::from(address));
            assert_eq!(receive_addresses[index as usize], address.to_string(), "receive address at {index} failed");
            //let address: String = hd_wallet.derive_change_address(index).await.unwrap().into();
            //assert_eq!(change_addresses[index as usize], address, "change address at {index} failed");
        }

        println!("receive_addresses: {receive_addresses:#?}");
    }
}
