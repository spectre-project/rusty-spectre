use spectre_bip32::Prefix;
use std::{fmt, str::FromStr};

use crate::imports::*;

///
/// Extended public key (XPub).
///
/// This class allows accepts another XPub and and provides
/// functions for derivation of dependent child public keys.
///
/// Please note that Spectre extended public keys use `kpub` prefix.
///
/// @see {@link PrivateKeyGenerator}, {@link PublicKeyGenerator}, {@link XPrv}, {@link Mnemonic}
/// @category Wallet SDK
///
#[derive(Clone, CastFromJs)]
#[wasm_bindgen]
pub struct XPub {
    inner: ExtendedPublicKey<secp256k1::PublicKey>,
}

impl XPub {
    pub fn inner(&self) -> &ExtendedPublicKey<secp256k1::PublicKey> {
        &self.inner
    }
}

#[wasm_bindgen]
impl XPub {
    #[wasm_bindgen(constructor)]
    pub fn try_new(xpub: &str) -> Result<XPub> {
        let inner = ExtendedPublicKey::<secp256k1::PublicKey>::from_str(xpub)?;
        Ok(Self { inner })
    }

    #[wasm_bindgen(js_name=deriveChild)]
    pub fn derive_child(&self, chile_number: u32, hardened: Option<bool>) -> Result<XPub> {
        let chile_number = ChildNumber::new(chile_number, hardened.unwrap_or(false))?;
        let inner = self.inner.derive_child(chile_number)?;
        Ok(Self { inner })
    }

    #[wasm_bindgen(js_name=derivePath)]
    pub fn derive_path(&self, path: &JsValue) -> Result<XPub> {
        let path = DerivationPath::try_cast_from(path)?;
        let inner = self.inner.clone().derive_path(path.as_ref().into())?;
        Ok(Self { inner })
    }

    //#[wasm_bindgen(js_name = toString)]
    #[wasm_bindgen(js_name = intoString)]
    pub fn to_str(&self, prefix: &str) -> Result<String> {
        Ok(self.inner.to_string(Some(prefix.try_into()?)))
    }

    #[wasm_bindgen(js_name = toPublicKey)]
    pub fn public_key(&self) -> PublicKey {
        self.inner.public_key().into()
    }
}

impl From<ExtendedPublicKey<secp256k1::PublicKey>> for XPub {
    fn from(inner: ExtendedPublicKey<secp256k1::PublicKey>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "XPub | string")]
    pub type XPubT;
}

impl TryCastFromJs for XPub {
    type Error = Error;
    fn try_cast_from(value: impl AsRef<JsValue>) -> Result<Cast<Self>, Self::Error> {
        Self::resolve(&value, || {
            if let Some(xpub) = value.as_ref().as_string() {
                Ok(XPub::try_new(xpub.as_str())?)
            } else {
                Err(Error::InvalidXPub)
            }
        })
    }
}

pub struct NetworkTaggedXpub {
    pub xpub: ExtendedPublicKey<secp256k1::PublicKey>,
    pub network_id: NetworkId,
}
// impl NetworkTaggedXpub {

// }

impl fmt::Display for NetworkTaggedXpub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let obj: XPub = self.xpub.clone().into();
        write!(f, "{}", obj.inner.to_string(Some(Prefix::from(self.network_id))))
    }
}

type TaggedXpub = (ExtendedPublicKey<secp256k1::PublicKey>, NetworkId);

impl From<TaggedXpub> for NetworkTaggedXpub {
    fn from(value: TaggedXpub) -> Self {
        Self { xpub: value.0, network_id: value.1 }
    }
}
