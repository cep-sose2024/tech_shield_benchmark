use super::YubiKeyProvider;
use crate::common::{
    crypto::KeyUsage, error::SecurityModuleError, traits::module_provider::Provider,
};
use crate::hsm::{HsmProviderConfig, ProviderConfig};
use base64::{engine::general_purpose, Engine};
use std::str::Utf8Error;
use tracing::instrument;
use x509_cert::der::Encode;

use yubikey::{
    piv::{self, AlgorithmId, SlotId},
    Error, YubiKey,
};

const SLOTS: [u32; 20] = [
    0x005f_c10d,
    0x005f_c10e,
    0x005f_c10f,
    0x005f_c110,
    0x005f_c111,
    0x005f_c112,
    0x005f_c113,
    0x005f_c114,
    0x005f_c115,
    0x005f_c116,
    0x005f_c117,
    0x005f_c118,
    0x005f_c119,
    0x005f_c11a,
    0x005f_c11b,
    0x005f_c11c,
    0x005f_c11d,
    0x005f_c11e,
    0x005f_c11f,
    0x005f_c120,
];

/// Implements the `Provider` trait, providing cryptographic operations utilizing a YubiKey.
///
/// This implementation interacts with a YubiKey device for key management and cryptographic
/// operations.
impl Provider for YubiKeyProvider {
    /// Creates a new cryptographic key identified by the provider given key_id.
    ///
    /// This method creates a persisted cryptographic key using the specified algorithm
    /// and identifier, making it retrievable for future operations. The key is created
    /// with the specified key usages and stored in the YubiKey.
    ///
    /// # Arguments
    ///
    ///
    /// # Returns
    ///
    /// The generated Public Key will be stored in the Yubikey as Object with futher information
    /// A `Result` that, on success, contains `Ok()`.
    /// On failure, it returns a `yubikey::Error`.
    #[instrument]
    fn create_key(
        &mut self,
        key_id: &str,
        config: Box<dyn ProviderConfig>,
    ) -> Result<(), SecurityModuleError> {
        if let Some(hsm_config) = config.as_any().downcast_ref::<HsmProviderConfig>() {
            self.key_algo = Some(hsm_config.key_algorithm);
            self.key_usages = Some(hsm_config.key_usage.clone());

            let mut yubikey = self.yubikey;
            let mut usage: &str = "";
            let mut slot: u32;

            if !self.load_key(key_id, config).is_ok() {
                match self.key_usages {
                    SignEncrypt => {
                        match self.key_algo {
                            Rsa => {
                                match get_free_slot(self.yubikey) {
                                    Ok(free) => {
                                        self.slot_id = free;
                                    }
                                    Err(err) => {
                                        return Err(SecurityModuleError::InitializationError(
                                            "No free slot available".to_string(),
                                        ));
                                    }
                                }
                                usage = "encrypt";
                                let gen_key = piv::generate(
                                    &mut self.yubikey,
                                    // SlotId wird noch variabel gemacht, abhängig davon wie viele Slots benötigt werden
                                    self.slot_id,
                                    AlgorithmId::Rsa2048,
                                    yubikey::PinPolicy::Default,
                                    yubikey::TouchPolicy::Default,
                                );
                                match gen_key {
                                    Ok(_) => {
                                        let gen_key = gen_key.as_ref().unwrap().to_der().unwrap();
                                        let gen_key = general_purpose::STANDARD.encode(&gen_key);
                                        let gen_key = format!(
                                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                                        gen_key.trim()
                                    );
                                        self.pkey = gen_key;
                                    }
                                    Err(err) => return Err(SecurityModuleError::Hsm(err)),
                                }
                            }
                            Ecc => {
                                match get_free_slot(self.yubikey) {
                                    Ok(free) => {
                                        self.slot_id = free;
                                    }
                                    Err(err) => {
                                        return Err(SecurityModuleError::InitializationError(
                                            "No free slot available".to_string(),
                                        ));
                                    }
                                }
                                usage = "sign";
                                let gen_key = piv::generate(
                                    &mut self.yubikey,
                                    // SlotId wird noch variabel gemacht, abhängig davon wie viele Slots benötigt werden
                                    self.slot_id,
                                    AlgorithmId::EccP256,
                                    yubikey::PinPolicy::Default,
                                    yubikey::TouchPolicy::Default,
                                );
                                let mut generated;
                                match gen_key {
                                    Ok(_) => {
                                        let gen_key = gen_key.as_ref().unwrap().to_der().unwrap();
                                        let gen_key = general_purpose::STANDARD.encode(&gen_key);
                                        let gen_key = format!(
                                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                                        gen_key.trim()
                                    );
                                        self.pkey = gen_key;
                                    }
                                    Err(err) => return Err(SecurityModuleError::Hsm(err)),
                                }
                            }
                            _ => Err(SecurityModuleError::Hsm("Key Algorithm not supported")),
                        }
                    }

                    Decrypt => {
                        match self.key_algo {
                            Rsa => {
                                match get_free_slot(self.yubikey) {
                                    Ok(free) => {
                                        self.slot_id = free;
                                    }
                                    Err(err) => {
                                        return Err(SecurityModuleError::InitializationError(
                                            "No free slot available".to_string(),
                                        ));
                                    }
                                }
                                usage = "decrypt";
                                let gen_key = piv::generate(
                                    &mut self.yubikey,
                                    // SlotId wird noch variabel gemacht, abhängig davon wie viele Slots benötigt werden
                                    self.slot_id,
                                    AlgorithmId::Rsa2048,
                                    yubikey::PinPolicy::Default,
                                    yubikey::TouchPolicy::Default,
                                );
                                match gen_key {
                                    Ok(_) => {
                                        let gen_key = gen_key.as_ref().unwrap().to_der().unwrap();
                                        let gen_key = general_purpose::STANDARD.encode(&gen_key);
                                        let gen_key = format!(
                                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                                        gen_key.trim()
                                    );
                                        self.pkey = gen_key;
                                    }
                                    Err(err) => return Err(SecurityModuleError::Hsm(err)),
                                }
                            }
                            Ecc => {
                                // TODO, not tested, might work
                            }
                            _ => Err(SecurityModuleError::Hsm("Key Algorithm not supported")),
                        }
                    }

                    _ => Err(SecurityModuleError::Hsm("Key Usage not supported")),
                }
            } else {
                match self.key_usages {
                    SignEncrypt => match self.key_algo {
                        Rsa => {
                            slot = self.slot_id;
                            usage = "encrypt";
                            let gen_key = piv::generate(
                                &mut self.yubikey,
                                // SlotId wird noch variabel gemacht, abhängig davon wie viele Slots benötigt werden
                                self.slot_id,
                                AlgorithmId::Rsa2048,
                                yubikey::PinPolicy::Default,
                                yubikey::TouchPolicy::Default,
                            );
                            match gen_key {
                                Ok(_) => {
                                    let gen_key = gen_key.as_ref().unwrap().to_der().unwrap();
                                    let gen_key = general_purpose::STANDARD.encode(&gen_key);
                                    let gen_key = format!(
                                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                                        gen_key.trim()
                                    );
                                    self.pkey = gen_key;
                                }
                                Err(err) => return Err(SecurityModuleError::Hsm(err)),
                            }
                        }
                        Ecc => {
                            slot = self.slot_id;
                            usage = "sign";
                            let gen_key = piv::generate(
                                &mut self.yubikey,
                                // SlotId wird noch variabel gemacht, abhängig davon wie viele Slots benötigt werden
                                SlotId::Retired(slot_id),
                                AlgorithmId::EccP256,
                                yubikey::PinPolicy::Default,
                                yubikey::TouchPolicy::Default,
                            );
                            match gen_key {
                                Ok(_) => {
                                    let gen_key = gen_key.as_ref().unwrap().to_der().unwrap();
                                    let gen_key = general_purpose::STANDARD.encode(&gen_key);
                                    let gen_key = format!(
                                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                                        gen_key.trim()
                                    );
                                    self.pkey = gen_key;
                                }
                                Err(err) => return Err(SecurityModuleError::Hsm(err)),
                            }
                        }
                        _ => Err(SecurityModuleError::Hsm("Key Algorithm not supported")),
                    },

                    Decrypt => {
                        match self.key_algo {
                            Rsa => {
                                slot = self.slot_id;
                                usage = "decrypt";
                                let gen_key = piv::generate(
                                    &mut self.yubikey,
                                    // SlotId wird noch variabel gemacht, abhängig davon wie viele Slots benötigt werden
                                    self.slot_id,
                                    AlgorithmId::Rsa2048,
                                    yubikey::PinPolicy::Default,
                                    yubikey::TouchPolicy::Default,
                                );
                                match gen_key {
                                    Ok(_) => {
                                        let gen_key = gen_key.as_ref().unwrap().to_der().unwrap();
                                        let gen_key = general_purpose::STANDARD.encode(&gen_key);
                                        let gen_key = format!(
                                        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                                        gen_key.trim()
                                    );
                                        self.pkey = gen_key;
                                    }
                                    Err(err) => return Err(SecurityModuleError::Hsm(err)),
                                }
                            }
                            Ecc => {
                                // TODO, not tested, might work
                            }
                            _ => Err(SecurityModuleError::Hsm("Key Algorithm not supported")),
                        }
                    }

                    _ => Err(SecurityModuleError::Hsm("Key Usage not supported")),
                }
            }

            save_key_object(yubikey, usage, key_id, slot, &self.pkey);

            Ok(())
        } else {
            Err(SecurityModuleError::Hsm("Failed to get the Configurations"))
        }
    }

    /// Loads an existing cryptographic key identified by `key_id`.
    ///
    /// This method attempts to load a persisted cryptographic key by its identifier from the YubiKey.
    /// If successful, it sets the key usages and returns a handle to the key for further
    /// cryptographic operations.
    ///
    /// # Arguments
    ///
    /// * `key_id` - A string slice that uniquely identifies the key to be loaded.
    /// * `config` - A boxed `ProviderConfig` containing configuration details for the key.
    ///
    /// # Returns
    ///
    /// A `Result` that, on success, contains `Ok(())`, indicating that the key was loaded successfully.
    /// On failure, it returns a `SecurityModuleError`.
    #[instrument]
    fn load_key(
        &mut self,
        key_id: &str,
        config: Box<dyn ProviderConfig>,
    ) -> Result<(), SecurityModuleError> {
        let mut yubikey = self.yubikey;
        let mut found = false;
        for i in 10..19 {
            let data = yubikey.fetch_object(SLOTS[i]);
            let mut output: Vec<u8> = Vec::new();
            match data {
                Ok(data) => {
                    output = data.to_vec();
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                }
            }

            let data = output;
            match parse_slot_data(&data) {
                Ok((key_name, slot, usage, public_key)) => {
                    if key_name == key_id {
                        self.slot_id = SLOTS[i - 10];
                        self.key_usages = match usage.as_str() {
                            "sign" | "encrypt" => KeyUsage::SignEncrypt,
                            "decrypt" => KeyUsage::Decrypt,
                            _ => continue,
                        };
                        self.pkey = public_key;
                        found = true;
                        break;
                    }
                }
                Err(e) => {
                    println!("Error parsing slot data: {:?}", e);
                    continue; // Gehe zur nächsten Iteration, wenn ein Fehler beim Parsen auftritt
                }
            }
        }

        if !found {
            return Err(SecurityModuleError::Hsm("Key not found"));
        }

        Ok(())
    }

    /// Initializes the YubiKey module and returns a handle for cryptographic operations.
    ///
    /// This method initializes the YubiKey device and sets up the necessary environment
    /// for cryptographic operations.
    ///
    /// # Arguments
    ///
    /// * `key_algorithm` - The asymmetric encryption algorithm to be used for the key.
    /// * `hash` - An optional hash algorithm to be used with the key.
    /// * `key_usages` - A vector of `KeyUsage` values specifying the intended usages for the key.
    ///
    /// # Returns
    ///
    /// A `Result` that, on success, contains `Ok(())`, indicating that the module was initialized successfully.
    /// On failure, it returns a Yubikey based `Error`.
    #[instrument]
    fn initialize_module(&mut self) -> Result<(), SecurityModuleError> {
        let yubikey = YubiKey::open().map_err(|_| Error::NotFound).unwrap();
        let verify = yubikey
            .verify_pin("123456".as_ref())
            .map_err(|_| Error::WrongPin {
                tries: yubikey.get_pin_retries().unwrap(),
            });

        self.yubikey = yubikey;

        if verify.is_ok() {
            return Ok(());
        } else {
            return Err(SecurityModuleError::Hsm(
                "Failed to verify PIN, retries: {}",
                yubikey.get_pin_retries().unwrap(),
            ));
        }
    }

    // Halbfertiger Code, kann benutzt werden wenn PIN-Abfrage in App implementiert wird
    /*
    #[instrument]
    fn initialize_module(
        &mut self,
        key_algorithm: AsymmetricEncryption,
        sym_algorithm: Option<BlockCiphers>,
        hash: Option<Hash>,
        key_usage: Vec<KeyUsage>,
        input: &str,
    ) -> Result<device, SecurityModuleError> {
        // Opens a connection to the yubikey device
        loop {
            let yubikey = YubiKey::open();
            if yubikey.is_ok() {
                let verify = device.verify_pin(input);
                if verify.is_ok() {
                    //successful login
                    return device;
                } else {
                    let count = device.get_pin_retries().unwrap();
                    // TODO: Implement PUK handling
                    if count == 0 {
                        return yubiKey::Error::PinLocked;
                        /*  let puk;
                        let pin_neu;
                        let change_puk = device.unblock_pin(puk.as_ref(), pin_neu.as_ref());
                        if change_puk.is_ok() {
                            return device;
                            */
                    }
                    return yubikey::Errror::WrongPin;
                }
            }
        }
    }
    */
}

/// Saves the key object to the YubiKey device.
///
/// This method saves a object to the YubiKey device. The object is stored in a slot and represents
/// information about the key, such as the key name, slot, key usage, and public key. This information
/// belongs to a private key which is stored in a other Slot.
///
/// # Arguments
/// 'usage' - The key usage of the key object to be stored.
///
/// # Returns
///
/// The saved Object will be stored in the Yubikey on a free Retired slot as Object with futher information
/// A `Result` that, on success, contains `Ok()`.
/// On failure, it returns a `yubikey::Error`.
fn save_key_object(
    yubikey: YubiKey,
    usage: &str,
    key_id: &str,
    slot_id: u32,
    pkey: &str,
) -> Result<(), yubikey::Error> {
    let key_name = key_id;
    let slot = slot_id.to_string();
    let public_key = pkey;

    let total_length = key_name.len() + 1 + slot.len() + 1 + usage.len() + 1 + public_key.len();
    let mut data = vec![0u8; total_length];
    let data_slice: &mut [u8] = &mut data;

    let mut offset = 0;
    data_slice[offset..offset + key_name.len()].copy_from_slice(key_name.as_bytes());
    offset += key_name.len();
    data_slice[offset] = 0;
    offset += 1;

    data_slice[offset..offset + slot.len()].copy_from_slice(slot.as_bytes());
    offset += slot.len();
    data_slice[offset] = 0;
    offset += 1;

    data_slice[offset..offset + usage.len()].copy_from_slice(usage.as_bytes());
    offset += usage.len();
    data_slice[offset] = 0;
    offset += 1;

    data_slice[offset..offset + public_key.len()].copy_from_slice(public_key.as_bytes());

    let saved = yubikey.save_object(slot_id, data_slice);
    match saved {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}

/// parses the u8 Data to different Key-Information Strings
///
/// This method creates a persisted cryptographic key using the specified algorithm
/// and identifier, making it retrievable for future operations. The key is created
/// with the specified key usages and stored in the YubiKey.
///
/// # Arguments
///
///
/// # Returns
///
/// A `Result` that, on success, contains `Ok(key_name, slot, key_usage, public_key)` where the individual information is given.
/// On failure, it returns a `Utf8Error`.
fn parse_slot_data(data: &[u8]) -> Result<(String, String, String, String), Utf8Error> {
    let parts: Vec<&[u8]> = data.split(|&x| x == 0).collect();
    let key_name = std::str::from_utf8(
        parts
            .get(0)
            .ok_or(Utf8Error::from_bytes_without_nul(data))?,
    )?
    .to_string();
    let slot = std::str::from_utf8(
        parts
            .get(1)
            .ok_or(Utf8Error::from_bytes_without_nul(data))?,
    )?
    .to_string();
    let usage = std::str::from_utf8(
        parts
            .get(2)
            .ok_or(Utf8Error::from_bytes_without_nul(data))?,
    )?
    .to_string();
    let public_key = std::str::from_utf8(
        parts
            .get(3)
            .ok_or(Utf8Error::from_bytes_without_nul(data))?,
    )?
    .to_string();

    Ok((key_name, slot, usage, public_key))
}

/// Gets a free slot for storing a key object.
///
/// This method goes through the available slots on the YubiKey and returns the first free slot
///
/// # Arguments
///
///
/// # Returns
///
/// A `Result` that, on failure, returns the first free slot.
/// On Success, it returns that no more free slots are available.
fn get_free_slot(yubikey: YubiKey) -> Result<u32, Error> {
    for i in 10..19 {
        let data = yubikey.fetch_object(SLOTS[i]);
        let mut output: Vec<u8> = Vec::new();
        match data {
            Ok(data) => {
                output = data.to_vec();
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }

        let data = output;
        match parse_slot_data(&data) {
            Ok(_) => {
                continue;
            }
            Err(_) => SLOTS[i - 10],
        }
    }
    Ok("No free slot available")
}
