// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later
//! # embassy-rp-hc05
//!
//! Driver async `no_std` pour le module Bluetooth **HC-05** testé sur
//! microcontrôleur **RP2040** / **RP235x**, basé sur le framework [Embassy](https://embassy.dev).
//!
//! ## Câblage minimal
//!
//! ```text
//! RP2040          HC-05
//! ──────          ─────
//! TX (GP0)  ───►  RX
//! RX (GP1)  ◄───  TX
//! 3.3V      ───►  VCC  (ou 5V selon module)
//! GND       ───►  GND
//! GP2 (opt) ◄───  STATE  (HIGH = connecté)
//! ```
//!
//! ## Utilisation rapide
//!
//! ```rust,ignore
//! let uart = Uart::new(p.UART0, p.PIN_0, p.PIN_1, Irqs, p.DMA_CH0, p.DMA_CH1, uart_config);
//! let mut bt = BluetoothHandler::new(uart, None);
//! bt.send_line("Hello!").await.unwrap();
//! ```

#![no_std]
#![forbid(unsafe_code)]

use embassy_rp::gpio::Input;
use embassy_rp::uart::{Async, Error as UartError, Uart};

/// Erreurs possibles du driver HC-05.
#[derive(Debug)]
pub enum BluetoothError {
    /// Erreur UART sous-jacente propagée depuis Embassy.
    Uart(UartError),
}

impl From<UartError> for BluetoothError {
    fn from(e: UartError) -> Self {
        BluetoothError::Uart(e)
    }
}

/// Driver principal pour le module HC-05.
///
/// Encapsule un [`Uart`] Embassy en mode asynchrone et, optionnellement,
/// une pin `STATE` pour détecter la connexion Bluetooth active.
pub struct BluetoothHandler<'d> {
    uart: Uart<'d, Async>,
    state_pin: Option<Input<'d>>,
}

impl<'d> BluetoothHandler<'d> {
    /// Crée un nouveau `BluetoothHandler`.
    ///
    /// # Arguments
    ///
    /// * `uart`      – Instance [`Uart`] Embassy configurée (baudrate 9600 par défaut pour HC-05).
    /// * `state_pin` – Pin `STATE` du HC-05 (`Some(pin)`) ou `None` si non câblée.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// let mut bt = BluetoothHandler::new(uart, Some(state_input));
    /// ```
    pub fn new(uart: Uart<'d, Async>, state_pin: Option<Input<'d>>) -> Self {
        Self { uart, state_pin }
    }

    /// Retourne `true` si le HC-05 indique une connexion Bluetooth active.
    ///
    /// - Si la pin `STATE` est câblée : lit son état logique (`HIGH` = connecté).
    /// - Si aucune pin n'est fournie : suppose toujours connecté (`true`).
    ///
    /// La pin `STATE` du HC-05 passe à `HIGH` (~3.3 V) quand un appareil
    /// est appairé et connecté.
    pub fn is_connected(&self) -> bool {
        match &self.state_pin {
            Some(pin) => pin.is_high(),
            None => true,
        }
    }

    /// Envoie une chaîne de caractères brute via UART.
    ///
    /// # Erreurs
    ///
    /// Retourne [`BluetoothError::Uart`] en cas d'échec de transmission.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send("OK").await?;
    /// ```
    pub async fn send(&mut self, message: &str) -> Result<(), BluetoothError> {
        self.uart
            .write(message.as_bytes())
            .await
            .map_err(BluetoothError::Uart)
    }

    /// Envoie un entier signé 16 bits (`i16`) converti en texte ASCII.
    ///
    /// Utilise [`itoa`] pour la conversion sans allocation.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send_i16(-1234).await?;  // envoie "-1234"
    /// ```
    pub async fn send_i16(&mut self, val: i16) -> Result<(), BluetoothError> {
        let mut buffer = itoa::Buffer::new();
        let text = buffer.format(val);
        self.send(text).await
    }

    /// Envoie un entier non signé 16 bits (`u16`) converti en texte ASCII.
    ///
    /// Pratique pour envoyer des valeurs ADC brutes (0–4095 ou 0–16383).
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send_u16(3012).await?;  // envoie "3012"
    /// ```
    pub async fn send_u16(&mut self, val: u16) -> Result<(), BluetoothError> {
        let mut buffer = itoa::Buffer::new();
        let text = buffer.format(val);
        self.send(text).await
    }

    /// Envoie un entier non signé 32 bits (`u32`) converti en texte ASCII.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send_u32(123456).await?;
    /// ```
    pub async fn send_u32(&mut self, val: u32) -> Result<(), BluetoothError> {
        let mut buffer = itoa::Buffer::new();
        let text = buffer.format(val);
        self.send(text).await
    }

    /// Envoie un message suivi d'un retour chariot `\r\n`.
    ///
    /// Equivalent à [`send`](Self::send) + `"\r\n"`. Utile pour les
    /// terminaux série ou les parsers ligne-par-ligne côté récepteur.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send_line("Bonjour").await?;  // envoie "Bonjour\r\n"
    /// ```
    pub async fn send_line(&mut self, message: &str) -> Result<(), BluetoothError> {
        self.send(message).await?;
        self.send("\r\n").await
    }

    /// Envoie un entier `i16` suivi d'un retour chariot `\r\n`.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send_i16_line(-42).await?;  // envoie "-42\r\n"
    /// ```
    pub async fn send_i16_line(&mut self, val: i16) -> Result<(), BluetoothError> {
        self.send_i16(val).await?;
        self.send("\r\n").await
    }

    /// Envoie un entier `u16` suivi d'un retour chariot `\r\n`.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// bt.send_u16_line(4095).await?;  // envoie "4095\r\n"
    /// ```
    pub async fn send_u16_line(&mut self, val: u16) -> Result<(), BluetoothError> {
        self.send_u16(val).await?;
        self.send("\r\n").await
    }

    /// Lit des octets depuis l'UART dans le buffer fourni.
    ///
    /// Bloque jusqu'à ce que le buffer soit rempli ou qu'une erreur survienne.
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// let mut buf = [0u8; 32];
    /// bt.read(&mut buf).await?;
    /// ```
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), BluetoothError> {
        self.uart
            .read(buffer)
            .await
            .map_err(BluetoothError::Uart)
    }

    
    /// Lit une ligne complète depuis le module Bluetooth.
    ///
    /// Cette méthode remplit le buffer fourni octet par octet jusqu'à ce que :
    /// 1. Le caractère de saut de ligne `\n` (LF) soit détecté.
    /// 2. Le buffer soit entièrement rempli (pour éviter tout débordement).
    ///
    /// # Fonctionnement asynchrone
    /// 
    /// Contrairement à `read`, cette méthode est réactive : elle retourne dès que la 
    /// phrase est terminée, même si le buffer est beaucoup plus grand que le message.
    ///
    /// # Arguments
    ///
    /// * `buf` Un buffer mutable pour stocker la ligne reçue.
    ///
    /// # Retour
    ///
    /// Retourne `Ok(usize)` représentant le nombre d'octets réellement écrits dans 
    /// le buffer (incluant le caractère `\n` s'il a été trouvé).
    ///
    /// # Exemple
    ///
    /// ```rust,ignore
    /// let mut command_buf = [0u8; 64];
    /// if let Ok(len) = bt.read_line(&mut command_buf).await {
    ///     let command = core::str::from_utf8(&command_buf[..len]).unwrap_or("");
    ///     if command.contains("START") {
    ///         // Activer le système...
    ///     }
    /// }
    /// ```
    pub async fn read_line(&mut self, buf: &mut [u8]) -> Result<usize, BluetoothError> {
        let mut i = 0;
        while i < buf.len() {
            let mut byte = [0u8; 1];
            // On utilise l'opérateur '?' pour propager l'erreur UART si elle survient
            self.uart.read(&mut byte).await?; 
            
            buf[i] = byte[0];
            
            // Détection du caractère de fin de ligne (Line Feed)
            if byte[0] == b'\n' {
                return Ok(i + 1);
            }
            i += 1;
        }
        // Retourne la taille remplie si le buffer est plein avant d'avoir trouvé '\n'
        Ok(i)
    }
}